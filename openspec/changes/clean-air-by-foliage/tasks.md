## Implementation (unclaimed)

- [ ] 1.1 `air` block in flora profiles (scrub oak, reeds, desert brush) with starting radii and strengths; validate on load.
- [ ] 1.2 `stubby_core::AirCleaner`; flora binders write it, its strength easing with growth and bloom.
- [ ] 2.1 `stubby_world`: `light_quality_at(point)` over cleaners (D1); `CameraLightQuality` follows it at the camera. Tests: locality, saturation, a dying grove fouling the air.
- [ ] 2.2 Mine productivity (`ProductivityFactor::LightQuality`) samples it at the mine.
- [ ] 3.1 Ground tint and flora emission sample it per place (was `add-desert-opening` 3.7).
- [ ] 4.1 Bounded review: grow a grove, walk from bleak desert into it at noon and at night; no visible steps.
