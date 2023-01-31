#include <stdint.h>
#include <Windows.h>
#include <Psapi.h>

typedef enum {
    OK = 0,
    READ_STEAM_REGISTRY,
    LAUNCH_STEAM,
    WAIT_STEAM_EXIT
} phase_t;

typedef struct {
    phase_t type;
    DWORD win_code;
} result_t;

#define SUCCESS ((result_t){OK,ERROR_SUCCESS})

typedef struct {
    wchar_t len;
    wchar_t path[MAX_PATH];
} steam_t;

result_t steam_init(steam_t *steam) {
    DWORD len = sizeof(steam->path);
    const LSTATUS result = RegGetValueW(
        HKEY_CURRENT_USER,
        L"SOFTWARE\\Valve\\Steam",
        L"SteamExe",
        RRF_RT_REG_SZ,
        NULL,
        &steam->path,
        &len
    );
    steam->len = len / sizeof(wchar_t);
    return result == ERROR_SUCCESS
        ? SUCCESS
        : (result_t){READ_STEAM_REGISTRY, result};
}

static result_t steam_launch_args(steam_t const *steam, wchar_t *args, PROCESS_INFORMATION *process) {
    STARTUPINFOW startup = {0};
    startup.cb = sizeof(startup);
    const BOOL launched = CreateProcessW(
        steam->path,
        args,
        NULL, NULL,
        FALSE,
        CREATE_NEW_PROCESS_GROUP,
        NULL,
        NULL,
        &startup,
        process
    );
    return launched ? SUCCESS : (result_t){LAUNCH_STEAM, GetLastError()};
}

result_t steam_shutdown(steam_t const *steam) {
    PROCESS_INFORMATION process;
    wchar_t args[] = L"-shutdown";
    const result_t launch_result = steam_launch_args(steam, args, &process);
    if (launch_result.type != OK && process.hProcess) {
        CloseHandle(process.hThread);
        CloseHandle(process.hProcess);
        return launch_result;
    }
    DWORD wait_result = WaitForSingleObject(process.hProcess, INFINITE);
    CloseHandle(process.hThread);
    CloseHandle(process.hProcess);
    DWORD pid = 0;
    steam_is_running_xxx(&pid);
    return wait_result == WAIT_FAILED
        ? (result_t){WAIT_STEAM_EXIT, GetLastError()}
        : SUCCESS;
}

result_t steam_launch(steam_t const *steam) {
    PROCESS_INFORMATION process;
    const result_t result = steam_launch_args(steam, NULL, &process);
    CloseHandle(process.hThread);
    CloseHandle(process.hProcess);
    return result;
}