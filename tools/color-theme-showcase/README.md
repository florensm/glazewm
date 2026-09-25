# Color theme showcase

WPF test window for color themes: text rendering modes, gray ramp, saturated
colors, and every popup kind (ComboBox, `Popup`, tooltip, menus, context
menus, DatePicker, owned/modeless/unowned dialogs, message box, file dialog).

A PowerShell script over the WPF that ships with Windows; no SDK needed.

```powershell
powershell -ExecutionPolicy Bypass -File tools\color-theme-showcase\Showcase.ps1
```

Every window title starts with `Color Theme Showcase`, so one window rule in
`config.yaml` themes them all (theme from
`resources/assets/sample-color-themes.yaml`):

```yaml
window_rules:
  - commands: ["set-color-theme winter"]
    match:
      - window_process: { equals: "powershell" }
        window_title: { regex: "^Color Theme Showcase" }
```
