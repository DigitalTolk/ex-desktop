# Icon Composer layers

Import these SVG files into Apple Icon Composer to create `AppIcon.icon`.

Use a 1024 x 1024 canvas, keep the files in numeric order from back to front, and let Icon Composer apply the Liquid Glass material. Do not add a baked background, blur, shadow, or bubble stroke in the SVGs.

After saving `src-tauri/icons/AppIcon.icon`, add `"icons/AppIcon.icon"` to `bundle.icon` in `src-tauri/tauri.conf.json`. Tauri 2.11 can compile `.icon` directories into macOS `Assets.car`, but this requires Xcode 26+ selected with `xcode-select`, not only Command Line Tools.
