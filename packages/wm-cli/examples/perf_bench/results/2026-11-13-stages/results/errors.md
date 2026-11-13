# errors.log warnings per mode (errlines.py), clean runs

## resize-load0

### `nopeeks`

- 72× WARN glazewm::commands::general::platform_sync: Failed to set focus: The operation completed successfully. (<hwnd>)
- 18× WARN glazewm::commands::general::platform_sync: Failed to set focus: The system cannot find the file specified. (<hwnd>)
- 12× WARN glazewm::ipc_server: Error handling connection: WebSocket error: WebSocket protocol error: Connection reset without closing handsha
- 4× WARN glazewm: GLAZEWM_ISOLATE changes how moving windows are drawn: Isolation { no_backdrop: false, no_ring: false, live: fa
- 1× WARN wm_platform::platform_impl::platform::native_window: Window <hwnd> (Notepad, Notepad) can't be cloaked, as the shell has no view of it (e.g. another window owns it

### `nopeeks+nokeepoverlays`

- 87× WARN glazewm::commands::general::platform_sync: Failed to set focus: The operation completed successfully. (<hwnd>)
- 12× WARN glazewm::ipc_server: Error handling connection: WebSocket error: WebSocket protocol error: Connection reset without closing handsha
- 4× WARN glazewm: GLAZEWM_ISOLATE changes how moving windows are drawn: Isolation { no_backdrop: false, no_ring: false, live: fa
- 2× WARN glazewm::commands::general::platform_sync: Failed to set focus: The system cannot find the file specified. (<hwnd>)

### `default`

- 84× WARN glazewm::commands::general::platform_sync: Failed to set focus: The operation completed successfully. (<hwnd>)
- 12× WARN glazewm::ipc_server: Error handling connection: WebSocket error: WebSocket protocol error: Connection reset without closing handsha

### `noasynchandback`

- 80× WARN glazewm::commands::general::platform_sync: Failed to set focus: The operation completed successfully. (<hwnd>)
- 19× WARN glazewm::commands::general::platform_sync: Synchronous reposition for Window(id=WindowId(#), process=FPilot, class=File Pilot) took #ms -- likely blocked
- 15× WARN glazewm::ipc_server: Error handling connection: WebSocket error: WebSocket protocol error: Connection reset without closing handsha
- 14× WARN glazewm::commands::general::platform_sync: Synchronous reposition for Window(id=WindowId(#), process=WindowsTerminal, class=CASCADIA_HOSTING_WINDOW_CLASS
- 4× WARN glazewm: GLAZEWM_ISOLATE changes how moving windows are drawn: Isolation { no_backdrop: false, no_ring: false, live: fa
- 4× WARN glazewm::commands::general::platform_sync: Synchronous reposition for Window(id=WindowId(#), process=Notepad, class=Notepad) took #ms -- likely blocked o
- 4× WARN glazewm::animation::manager: Synchronous move of window <hwnd> took #ms; moving it asynchronously for #s.
- 4× WARN glazewm::commands::general::platform_sync: Failed to set focus: The system cannot find the file specified. (<hwnd>)

## switch-load0

### `nopeeks`

- 34× WARN glazewm::commands::general::platform_sync: Failed to set focus: The operation completed successfully. (<hwnd>)
- 15× WARN glazewm::commands::general::platform_sync: Failed to set focus: The system cannot find the file specified. (<hwnd>)
- 12× WARN glazewm::ipc_server: Error handling connection: WebSocket error: WebSocket protocol error: Connection reset without closing handsha
- 4× WARN glazewm: GLAZEWM_ISOLATE changes how moving windows are drawn: Isolation { no_backdrop: false, no_ring: false, live: fa
- 1× WARN wm_platform::platform_impl::platform::native_window: Window <hwnd> (Notepad, Notepad) can't be cloaked, as the shell has no view of it (e.g. another window owns it

### `default`

- 35× WARN glazewm::commands::general::platform_sync: Failed to set focus: The operation completed successfully. (<hwnd>)
- 12× WARN glazewm::ipc_server: Error handling connection: WebSocket error: WebSocket protocol error: Connection reset without closing handsha
- 9× WARN glazewm::commands::general::platform_sync: Failed to set focus: The system cannot find the file specified. (<hwnd>)

### `noslidepeeks`

- 33× WARN glazewm::commands::general::platform_sync: Failed to set focus: The operation completed successfully. (<hwnd>)
- 15× WARN glazewm::ipc_server: Error handling connection: WebSocket error: WebSocket protocol error: Connection reset without closing handsha
- 13× WARN glazewm::commands::general::platform_sync: Failed to set focus: The system cannot find the file specified. (<hwnd>)
- 4× WARN glazewm: GLAZEWM_ISOLATE changes how moving windows are drawn: Isolation { no_backdrop: false, no_ring: false, live: fa

## main-load0

### `main`

- 80× WARN wm_platform::platform_impl::platform::native_window: Failed to get window's frame position. Falling back to border position.
- 22× WARN glazewm::ipc_server: Error handling connection: WebSocket error: WebSocket protocol error: Connection reset without closing handsha
- 20× WARN glazewm::ipc_server: Failed to broadcast disconnection: channel closed
- 18× WARN wm_platform::native_border_overlay: Platform error: Border overlay SetWindowPos failed: Access is denied. (<hwnd>).
- 4× WARN glazewm::wm_state: Failed to redraw window on cleanup: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 4× WARN glazewm::wm_state: Failed to show window: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 3× WARN wm_platform::resize_session: ResizeSession commit for AnyDesk did not land within #ms -- uncloaking anyway (see COMMIT_CONFIRM_MAX_WAIT's d

### `nopeeks`

- 264× WARN wm_platform::platform_impl::platform::window_listener: Failed to send window event: channel closed.
- 20× WARN glazewm::ipc_server: Error handling connection: WebSocket error: WebSocket protocol error: Connection reset without closing handsha
- 8× WARN glazewm::ipc_server: Failed to broadcast disconnection: channel closed
- 4× WARN glazewm::wm_state: Failed to redraw window on cleanup: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 4× WARN glazewm::wm_state: Failed to show window: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 4× WARN glazewm: GLAZEWM_ISOLATE changes how moving windows are drawn: Isolation { no_backdrop: false, no_ring: false, live: fa

### `default`

- 20× WARN glazewm::ipc_server: Error handling connection: WebSocket error: WebSocket protocol error: Connection reset without closing handsha
- 5× WARN glazewm::wm_state: Failed to redraw window on cleanup: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 5× WARN glazewm::wm_state: Failed to show window: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 5× WARN glazewm::animation::peek_view: Failed to create a peek of Window(id=WindowId(#), process=AnyDesk, class=ad_win##

## resize-cpu8

### `nopeeks`

- 11× WARN glazewm::ipc_server: Error handling connection: WebSocket error: WebSocket protocol error: Connection reset without closing handsha
- 4× WARN glazewm::wm_state: Failed to redraw window on cleanup: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 4× WARN glazewm::wm_state: Failed to show window: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 4× WARN glazewm: GLAZEWM_ISOLATE changes how moving windows are drawn: Isolation { no_backdrop: false, no_ring: false, live: fa

### `nopeeks+nokeepoverlays`

- 12× WARN glazewm::ipc_server: Error handling connection: WebSocket error: WebSocket protocol error: Connection reset without closing handsha
- 4× WARN glazewm::wm_state: Failed to redraw window on cleanup: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 4× WARN glazewm::wm_state: Failed to show window: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 4× WARN glazewm: GLAZEWM_ISOLATE changes how moving windows are drawn: Isolation { no_backdrop: false, no_ring: false, live: fa

### `default`

- 12× WARN glazewm::ipc_server: Error handling connection: WebSocket error: WebSocket protocol error: Connection reset without closing handsha
- 4× WARN glazewm::wm_state: Failed to redraw window on cleanup: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 4× WARN glazewm::wm_state: Failed to show window: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 4× WARN glazewm::animation::peek_view: Failed to create a peek of Window(id=WindowId(#), process=AnyDesk, class=ad_win##

### `noasynchandback`

- 20× WARN glazewm::commands::general::platform_sync: Synchronous reposition for Window(id=WindowId(#), process=FPilot, class=File Pilot) took #ms -- likely blocked
- 15× WARN glazewm::ipc_server: Error handling connection: WebSocket error: WebSocket protocol error: Connection reset without closing handsha
- 5× WARN glazewm::wm_state: Failed to redraw window on cleanup: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 5× WARN glazewm::wm_state: Failed to show window: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 5× WARN glazewm::animation::peek_view: Failed to create a peek of Window(id=WindowId(#), process=AnyDesk, class=ad_win##
- 4× WARN glazewm: GLAZEWM_ISOLATE changes how moving windows are drawn: Isolation { no_backdrop: false, no_ring: false, live: fa
- 4× WARN glazewm::commands::general::platform_sync: Synchronous reposition for Window(id=WindowId(#), process=Notepad, class=Notepad) took #ms -- likely blocked o
- 4× WARN glazewm::animation::manager: Synchronous move of window <hwnd> took #ms; moving it asynchronously for #s.
- 4× WARN glazewm::commands::general::platform_sync: Synchronous reposition for Window(id=WindowId(#), process=chrome, class=Chrome_WidgetWin_#) took #ms -- likely
- 4× WARN glazewm::commands::general::platform_sync: Synchronous reposition for Window(id=WindowId(#), process=WindowsTerminal, class=CASCADIA_HOSTING_WINDOW_CLASS

## switch-cpu8

### `nopeeks`

- 11× WARN glazewm::ipc_server: Error handling connection: WebSocket error: WebSocket protocol error: Connection reset without closing handsha
- 4× WARN glazewm::wm_state: Failed to redraw window on cleanup: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 4× WARN glazewm::wm_state: Failed to show window: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 4× WARN glazewm: GLAZEWM_ISOLATE changes how moving windows are drawn: Isolation { no_backdrop: false, no_ring: false, live: fa

### `default`

- 12× WARN glazewm::ipc_server: Error handling connection: WebSocket error: WebSocket protocol error: Connection reset without closing handsha
- 4× WARN glazewm::wm_state: Failed to redraw window on cleanup: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 4× WARN glazewm::wm_state: Failed to show window: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 4× WARN glazewm::animation::peek_view: Failed to create a peek of Window(id=WindowId(#), process=AnyDesk, class=ad_win##

### `noslidepeeks`

- 15× WARN glazewm::ipc_server: Error handling connection: WebSocket error: WebSocket protocol error: Connection reset without closing handsha
- 5× WARN glazewm::wm_state: Failed to redraw window on cleanup: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 5× WARN glazewm::wm_state: Failed to show window: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 5× WARN glazewm::animation::peek_view: Failed to create a peek of Window(id=WindowId(#), process=AnyDesk, class=ad_win##
- 4× WARN glazewm: GLAZEWM_ISOLATE changes how moving windows are drawn: Isolation { no_backdrop: false, no_ring: false, live: fa

## main-cpu8

### `main`

- 76× WARN wm_platform::platform_impl::platform::native_window: Failed to get window's frame position. Falling back to border position.
- 22× WARN glazewm::ipc_server: Error handling connection: WebSocket error: WebSocket protocol error: Connection reset without closing handsha
- 20× WARN glazewm::ipc_server: Failed to broadcast disconnection: channel closed
- 6× WARN wm_platform::native_border_overlay: Platform error: Border overlay SetWindowPos failed: Access is denied. (<hwnd>).
- 1× WARN glazewm::wm_state: Failed to redraw window on cleanup: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 1× WARN glazewm::wm_state: Failed to show window: Windows(Error { code: HRESULT(<hwnd>), message: "Access is denied." })
- 1× WARN wm_platform::resize_session: ResizeSession commit for AnyDesk did not land within #ms -- uncloaking anyway (see COMMIT_CONFIRM_MAX_WAIT's d

### `nopeeks`

- 175× WARN wm_platform::platform_impl::platform::window_listener: Failed to send window event: channel closed.
- 20× WARN glazewm::ipc_server: Error handling connection: WebSocket error: WebSocket protocol error: Connection reset without closing handsha
- 8× WARN glazewm::ipc_server: Failed to broadcast disconnection: channel closed
- 4× WARN glazewm: GLAZEWM_ISOLATE changes how moving windows are drawn: Isolation { no_backdrop: false, no_ring: false, live: fa

### `default`

- 20× WARN glazewm::ipc_server: Error handling connection: WebSocket error: WebSocket protocol error: Connection reset without closing handsha
