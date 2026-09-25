# Color theme showcase

WPF test window for color themes: text rendering modes, gray ramp, saturated
colors, and every popup kind (ComboBox, `Popup`, tooltip, menus, context
menus, DatePicker, owned/modeless/unowned dialogs, message box, file dialog).

Needs the [.NET 8 SDK](https://dotnet.microsoft.com/download/dotnet/8.0).

```powershell
cd tools/color-theme-showcase
dotnet run
```

Theme it with a window rule in `config.yaml` (theme from
`resources/assets/sample-color-themes.yaml`):

```yaml
window_rules:
  - commands: ["set-color-theme winter"]
    match:
      - window_process: { equals: "ColorThemeShowcase" }
```
