import AppKit
import CoreGraphics
import Foundation

let repoRoot = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
let iconsDir = repoRoot.appendingPathComponent("src-tauri/icons", isDirectory: true)

func savePNG(_ bitmap: NSBitmapImageRep, to url: URL) throws {
    guard let png = bitmap.representation(using: .png, properties: [:]) else {
        throw NSError(domain: "IconGeneration", code: 1)
    }
    try png.write(to: url)
}

func makeImage(size: Int, draw: (CGContext, CGFloat) -> Void) -> NSBitmapImageRep {
    let bitmap = NSBitmapImageRep(
        bitmapDataPlanes: nil,
        pixelsWide: size,
        pixelsHigh: size,
        bitsPerSample: 8,
        samplesPerPixel: 4,
        hasAlpha: true,
        isPlanar: false,
        colorSpaceName: .deviceRGB,
        bytesPerRow: 0,
        bitsPerPixel: 0
    )!
    bitmap.size = NSSize(width: size, height: size)
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: bitmap)
    let context = NSGraphicsContext.current!.cgContext
    context.clear(CGRect(x: 0, y: 0, width: CGFloat(size), height: CGFloat(size)))
    draw(context, CGFloat(size) / 64.0)
    NSGraphicsContext.restoreGraphicsState()
    return bitmap
}

func speechBubblePath(scale: CGFloat) -> CGPath {
    let path = CGMutablePath()
    func p(_ x: CGFloat, _ y: CGFloat) -> CGPoint {
        CGPoint(x: x * scale, y: (64.0 - y) * scale)
    }

    path.move(to: p(12, 12))
    path.addLine(to: p(52, 12))
    path.addQuadCurve(to: p(58, 18), control: p(58, 12))
    path.addLine(to: p(58, 40))
    path.addQuadCurve(to: p(52, 46), control: p(58, 46))
    path.addLine(to: p(28, 46))
    path.addLine(to: p(18, 56))
    path.addLine(to: p(18, 46))
    path.addLine(to: p(12, 46))
    path.addQuadCurve(to: p(6, 40), control: p(6, 46))
    path.addLine(to: p(6, 18))
    path.addQuadCurve(to: p(12, 12), control: p(6, 12))
    path.closeSubpath()
    return path
}

func drawCircle(_ context: CGContext, x: CGFloat, y: CGFloat, radius: CGFloat, scale: CGFloat, color: CGColor) {
    context.setFillColor(color)
    context.fillEllipse(in: CGRect(
        x: (x - radius) * scale,
        y: (64.0 - y - radius) * scale,
        width: radius * 2 * scale,
        height: radius * 2 * scale
    ))
}

func appIcon(size: Int) -> NSBitmapImageRep {
    makeImage(size: size) { context, scale in
        context.setShadow(
            offset: CGSize(width: 0, height: -1.1 * scale),
            blur: 2.6 * scale,
            color: NSColor.black.withAlphaComponent(0.18).cgColor
        )
        let path = speechBubblePath(scale: scale)
        context.addPath(path)
        context.setFillColor(NSColor.white.withAlphaComponent(0.92).cgColor)
        context.fillPath()
        context.setShadow(offset: .zero, blur: 0, color: nil)

        let dark = NSColor(red: 0.20, green: 0.18, blue: 0.18, alpha: 0.86).cgColor
        let accent = NSColor(red: 0.871, green: 0.365, blue: 0.514, alpha: 1).cgColor
        drawCircle(context, x: 20, y: 29, radius: 3, scale: scale, color: dark)
        drawCircle(context, x: 32, y: 29, radius: 3, scale: scale, color: dark)
        drawCircle(context, x: 44, y: 29, radius: 3, scale: scale, color: accent)
    }
}

func trayIcon(size: Int, badge: Bool) -> NSBitmapImageRep {
    makeImage(size: size) { context, scale in
        let path = speechBubblePath(scale: scale)
        context.addPath(path)
        context.setStrokeColor(NSColor.black.cgColor)
        context.setLineWidth(3 * scale)
        context.setLineCap(.round)
        context.setLineJoin(.round)
        context.strokePath()

        let black = NSColor.black.cgColor
        drawCircle(context, x: 20, y: 29, radius: 3, scale: scale, color: black)
        drawCircle(context, x: 32, y: 29, radius: 3, scale: scale, color: black)
        drawCircle(context, x: 44, y: 29, radius: 3, scale: scale, color: black)
        if badge {
            drawCircle(context, x: 52, y: 50, radius: 7, scale: scale, color: black)
        }
    }
}

try FileManager.default.createDirectory(at: iconsDir, withIntermediateDirectories: true)
try savePNG(appIcon(size: 1024), to: iconsDir.appendingPathComponent("app-icon.png"))
try savePNG(appIcon(size: 512), to: iconsDir.appendingPathComponent("icon.png"))
try savePNG(appIcon(size: 32), to: iconsDir.appendingPathComponent("32x32.png"))
try savePNG(appIcon(size: 128), to: iconsDir.appendingPathComponent("128x128.png"))
try savePNG(appIcon(size: 256), to: iconsDir.appendingPathComponent("128x128@2x.png"))
try savePNG(trayIcon(size: 64, badge: false), to: iconsDir.appendingPathComponent("tray-template.png"))
try savePNG(trayIcon(size: 64, badge: true), to: iconsDir.appendingPathComponent("tray-badge-template.png"))
