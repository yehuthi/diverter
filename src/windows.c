#include <stdint.h>
#include <Windows.h>
#include <Psapi.h>
#include <Shlwapi.h>
#include <ctype.h>

// note: on change, sync the Phase enum in steam.rs
typedef enum {
    OK = 0,
    READ_STEAM_REGISTRY,
    WRITE_STEAM_REGISTRY,
    CANONICALIZE_STEAM_PATH,
    LAUNCH_STEAM,
    WAIT_STEAM_EXIT,
    ENUM_PROCESSES,
    KILL_STEAM
} phase_t;

typedef struct {
    phase_t type;
    DWORD win_code;
} result_t;

#define SUCCESS ((result_t){OK,ERROR_SUCCESS})
#define FAILURE(type) ((result_t){type,GetLastError()})

typedef struct {
    /// path length excluding NUL terminator.
    wchar_t len;
    /// lowercase path to the steam executable.
    wchar_t path[MAX_PATH];
} steam_t;

result_t steam_init(steam_t *steam) {
    DWORD size = sizeof(steam->path);
    const LSTATUS status = RegGetValueW(
        HKEY_CURRENT_USER,
        L"SOFTWARE\\Valve\\Steam",
        L"SteamExe",
        RRF_RT_REG_SZ,
        NULL,
        &steam->path,
        &size
    );
    if (status != ERROR_SUCCESS) return (result_t){READ_STEAM_REGISTRY, status};
    steam->len = size / sizeof(wchar_t) - /* NUL */ 1;
    for (size_t i = 0; i < steam->len; i++)
        steam->path[i] = steam->path[i] == '/' ? '\\' : towlower(steam->path[i]);
    return SUCCESS;
}

/// note 1: close the handles to the process info's process and thread after their use.
/// note 2: args needs to be writable.
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

result_t steam_launch_fast(steam_t const *steam) {
    PROCESS_INFORMATION process;
    wchar_t args[] = L"-noverifyfiles -noverifyfiles";
    // the flag is passed twice, because passing it once doesn't seem to work.
    // this might be because perhaps Steam ignores the first argument, expecting it to be its executable path.
    const result_t result = steam_launch_args(steam, args, &process);
    CloseHandle(process.hThread);
    CloseHandle(process.hProcess);
    return result;
}

/// @return dir length, excluding NUL
static size_t steam_dir_lowercase(steam_t const *steam, wchar_t out[MAX_PATH]) {
    const size_t dir_len = steam->len - (sizeof("steam.exe") - /* NUL */ 1);
    memcpy(out, steam->path, dir_len * sizeof(wchar_t));
    out[dir_len] = L'\0';
    return dir_len;
}

static uint8_t steam_path_is_ancestor(wchar_t* path, size_t path_len, wchar_t* dir_lowercase, size_t dir_len) {
    if (path_len < dir_len) return 0;
    for (size_t i = dir_len - 1; i != ((size_t)-1); i--)
        if (towlower(path[i]) != dir_lowercase[i]) return 0;
    return 1;
}

result_t steam_kill(steam_t const *steam, uint8_t killed) {
    DWORD pids[5000];
    DWORD bytes_len = 0;
    if (!EnumProcesses(pids, sizeof(pids), &bytes_len))
        return FAILURE(ENUM_PROCESSES);
    const size_t len = bytes_len / sizeof(DWORD);

    wchar_t dir[MAX_PATH];
    const size_t dir_len = steam_dir_lowercase(steam, dir);

    for (size_t i = 0; i < len; i++) {
        const DWORD pid = pids[i];
        const HANDLE process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_TERMINATE, FALSE, pid);
        if (process == NULL) continue;
        wchar_t path[MAX_PATH];
        DWORD path_len = sizeof(path) / sizeof(wchar_t);
        if (!QueryFullProcessImageNameW(process, 0, &path, &path_len)) goto next_process;
        if (steam_path_is_ancestor(path, path_len, dir, dir_len)) {
            if (!TerminateProcess(process, EXIT_SUCCESS)) {
                CloseHandle(process);
                return FAILURE(KILL_STEAM);
            }
        }

        next_process:
        CloseHandle(process);
    }
    return SUCCESS;
}

/// ensure username is lowercase and username_len includes NUL terminator
result_t steam_set_auto_login_user(const char* username, uint8_t username_len) {
    LSTATUS status = RegSetKeyValueA(
        HKEY_CURRENT_USER,
        "SOFTWARE\\Valve\\Steam",
        "AutoLoginUser",
        REG_SZ,
        username,
        username_len);
    return (status == ERROR_SUCCESS) ? SUCCESS : (result_t){WRITE_STEAM_REGISTRY, status};
}

/// ensure username is lowercase and username_len includes NUL terminator
result_t steam_get_auto_login_user(char* username, uint8_t *username_len) {
    DWORD len = username_len;
    LSTATUS status = RegGetValueA(
        HKEY_CURRENT_USER,
        "SOFTWARE\\Valve\\Steam",
        "AutoLoginUser",
        RRF_RT_REG_SZ,
        NULL,
        username,
        &len);
    *username_len = len;
    return (status == ERROR_SUCCESS) ? SUCCESS : (result_t){WRITE_STEAM_REGISTRY, status};
}