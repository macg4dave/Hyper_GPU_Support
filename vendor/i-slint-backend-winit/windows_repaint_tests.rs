// Regression coverage for the local Windows exposure-region backport.
// These tests use an in-memory renderer and never open a native window.
use super::windows_update_region;
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, PlatformError, WindowAdapter};
use slint::{ComponentHandle, PhysicalSize};
use std::rc::Rc;
use windows::Win32::Foundation::RECT;

#[test]
fn windows_exposure_region_respects_fractional_dpi() {
    let rect = RECT { left: 125, top: 250, right: 375, bottom: 500 };
    let logical = windows_update_region(rect, 1.25);
    assert_eq!(logical.origin, euclid::point2(100.0, 200.0));
    assert_eq!(logical.size, euclid::size2(200.0, 200.0));
}

#[test]
fn windows_exposure_repaints_latest_pixels_without_another_ui_change() {
    struct TestPlatform(Rc<MinimalSoftwareWindow>);
    impl Platform for TestPlatform {
        fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
            Ok(self.0.clone())
        }
    }

    slint::slint! {
        export component ExposureScene inherits Window {
            width: 100px;
            height: 100px;
            background: #123456;
            in property <color> marker: #ff0000;
            Rectangle { x: 0px; y: 60px; width: 20px; height: 20px; background: root.marker; }
        }
    }

    let window = MinimalSoftwareWindow::new(RepaintBufferType::ReusedBuffer);
    slint::platform::set_platform(Box::new(TestPlatform(window.clone()))).unwrap();
    let scene = ExposureScene::new().unwrap();
    window.set_size(PhysicalSize::new(100, 100));
    scene.show().unwrap();
    let mut buffer = vec![slint::Rgb8Pixel::default(); 100 * 100];
    assert!(window.draw_if_needed(|renderer| { renderer.render(&mut buffer, 100); }));

    // A UI update renders while the lower half is off-screen. The reusable
    // backing buffer has the latest content, but exposed screen pixels are lost.
    scene.set_marker(slint::Color::from_rgb_u8(0, 255, 0));
    assert!(window.draw_if_needed(|renderer| { renderer.render(&mut buffer, 100); }));
    let latest = buffer.clone();
    scene.window().request_redraw();
    assert!(window.draw_if_needed(|renderer| {
        let region = renderer.render(&mut buffer, 100);
        assert_eq!(region.bounding_box_size(), PhysicalSize::new(0, 0));
    }));

    buffer[50 * 100..].fill(slint::Rgb8Pixel::default());
    let exposed = windows_update_region(
        RECT { left: 0, top: 50, right: 100, bottom: 100 },
        1.0,
    );
    window.renderer().mark_dirty_region(exposed.into());
    scene.window().request_redraw();
    assert!(window.draw_if_needed(|renderer| {
        let region = renderer.render(&mut buffer, 100);
        assert_eq!(region.bounding_box_origin(), slint::PhysicalPosition::new(0, 50));
        assert_eq!(region.bounding_box_size(), PhysicalSize::new(100, 50));
    }));
    assert_eq!(buffer, latest);
}
