# Generated test pictures for the Pictures tab, as raw Bgr32 pixels, so the
# showcase needs no image files. Shown upscaled with linear filtering, they
# shade smoothly in both directions with a little grain, like photos.
#
# Pure PowerShell (no WPF), so it can be tested on its own. Math calls
# pass `0.0`, not `0`: `[Math]::Max(0, 0.6)` picks the integer overload and
# rounds.

# `Kind` is one of: landscape, object (on white), night, portrait.
function New-PicturePixels([int] $Width, [int] $Height, [string] $Kind, [int] $Seed) {
  $random = [Random]::new($Seed)
  $pixels = New-Object 'byte[]' ($Width * $Height * 4)

  # Coarse random grid, interpolated for soft variation.
  $gridWidth = 6
  $gridHeight = 5
  $grid = New-Object 'double[]' ($gridWidth * $gridHeight)
  for ($i = 0; $i -lt $grid.Length; $i++) { $grid[$i] = $random.NextDouble() }

  $phase = $random.NextDouble() * 6.28
  $aspect = $Width / $Height

  for ($y = 0; $y -lt $Height; $y++) {
    $v = $y / ($Height - 1)
    for ($x = 0; $x -lt $Width; $x++) {
      $u = $x / ($Width - 1)

      # Soft noise in [0, 1].
      $gx = $u * ($gridWidth - 1)
      $gy = $v * ($gridHeight - 1)
      $x0 = [Math]::Min([int][Math]::Floor($gx), $gridWidth - 2)
      $y0 = [Math]::Min([int][Math]::Floor($gy), $gridHeight - 2)
      $fx = $gx - $x0
      $fy = $gy - $y0
      $fx = $fx * $fx * (3 - 2 * $fx)
      $fy = $fy * $fy * (3 - 2 * $fy)
      $top = $grid[$y0 * $gridWidth + $x0] * (1 - $fx) + $grid[$y0 * $gridWidth + $x0 + 1] * $fx
      $bottom = $grid[($y0 + 1) * $gridWidth + $x0] * (1 - $fx) + $grid[($y0 + 1) * $gridWidth + $x0 + 1] * $fx
      $noise = $top * (1 - $fy) + $bottom * $fy

      # Film grain.
      $grain = ($random.NextDouble() - 0.5) * 14
      $isFlat = $false

      switch ($Kind) {
        'landscape' {
          $horizon = 0.55 + 0.07 * [Math]::Sin($u * 7 + $phase) + 0.06 * ($noise - 0.5)
          if ($v -lt $horizon) {
            $r = 60 + 150 * $v
            $g = 120 + 110 * $v
            $b = 210 + 30 * $v
            $sun = [Math]::Sqrt([Math]::Pow(($u - 0.78) * $aspect, 2) + [Math]::Pow($v - 0.22, 2))
            $glow = [Math]::Max(0.0, 1 - $sun / 0.22)
            $r += 90 * $glow
            $g += 70 * $glow
            $b -= 60 * $glow
          } else {
            $depth = ($v - $horizon) / (1 - $horizon)
            $r = 70 + 50 * $noise - 30 * $depth
            $g = 130 + 60 * $noise - 50 * $depth
            $b = 50 + 20 * $noise
          }
        }
        'object' {
          # A shaded ball and its shadow on a pure white background.
          $dx = ($u - 0.5) * $aspect
          $dy = $v - 0.45
          $distance = [Math]::Sqrt($dx * $dx + $dy * $dy)
          $radius = 0.3
          if ($distance -lt $radius) {
            $nx = $dx / $radius
            $ny = $dy / $radius
            $nz = [Math]::Sqrt([Math]::Max(0.0, 1 - $nx * $nx - $ny * $ny))
            # Wrapped diffuse light from the top left, and a highlight.
            $light = [Math]::Max(0.0, (-0.45 * $nx - 0.55 * $ny + 0.7 * $nz + 0.35) / 1.35)
            $shine = [Math]::Pow($light, 16) * 70
            $r = 60 + 150 * $light + $shine
            $g = 20 + 45 * $light + $shine
            $b = 25 + 40 * $light + $shine
          } else {
            $shadow = [Math]::Sqrt([Math]::Pow($dx / 1.4, 2) + [Math]::Pow(($v - 0.8) * 3, 2))
            if ($shadow -lt 0.3) {
              $shade = 255 - 70 * (1 - $shadow / 0.3)
              $r = $shade
              $g = $shade
              $b = $shade
            } else {
              $isFlat = $true
              $r = 255
              $g = 255
              $b = 255
            }
          }
        }
        'night' {
          $r = 12 + 30 * $v + 20 * $noise
          $g = 18 + 30 * $v + 20 * $noise
          $b = 45 + 50 * $v + 30 * $noise
          for ($k = 0; $k -lt 3; $k++) {
            $lu = 0.2 + 0.3 * $k
            $lv = 0.6 + 0.1 * [Math]::Sin($k + $phase)
            $lamp = [Math]::Sqrt([Math]::Pow(($u - $lu) * $aspect, 2) + [Math]::Pow($v - $lv, 2))
            $glow = [Math]::Max(0.0, 1 - $lamp / 0.18)
            $r += 230 * $glow * $glow
            $g += 170 * $glow * $glow
            $b += 60 * $glow * $glow
          }
        }
        'portrait' {
          $r = 90 + 80 * $u + 40 * $noise
          $g = 150 - 40 * $v + 30 * $noise
          $b = 160 + 50 * $v
          $dx = ($u - 0.5) * $aspect / 0.55
          $dy = ($v - 0.55) / 0.75
          $head = [Math]::Sqrt($dx * $dx + $dy * $dy)
          if ($head -lt 0.6) {
            $shade = 1 - 0.5 * [Math]::Pow($head / 0.6, 2)
            $r = (150 + 80 * $noise) * $shade + 40
            $g = (100 + 50 * $noise) * $shade + 25
            $b = (70 + 30 * $noise) * $shade + 15
            if ($dy -lt -0.25) {
              $r *= 0.35
              $g *= 0.3
              $b *= 0.3
            }
          }
        }
        default { throw "Unknown picture kind '$Kind'." }
      }

      if (-not $isFlat) {
        $r += $grain
        $g += $grain
        $b += $grain
      }

      $index = ($y * $Width + $x) * 4
      $pixels[$index] = [byte][Math]::Max(0.0, [Math]::Min(255.0, $b))
      $pixels[$index + 1] = [byte][Math]::Max(0.0, [Math]::Min(255.0, $g))
      $pixels[$index + 2] = [byte][Math]::Max(0.0, [Math]::Min(255.0, $r))
      $pixels[$index + 3] = 255
    }
  }

  , $pixels
}
