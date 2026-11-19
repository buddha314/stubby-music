## 1. Developer audio control

- [x] 1.1 Add the OpenSpec requirement and developer-console page panel using the existing HTML and BRP connection.
- [x] 1.2 Add a dev-only reflected settings resource and synchronize changes to the audio engine.
- [x] 1.3 Add an audio command and atomic wet-mix control that updates current and future AudioNimbus emitters.
- [x] 1.4 (claimed: Codex) Add focused tests for wet-mix range handling and callback-safe scalar application. Tests reject non-finite values, verify clamping, and apply a changed wet gain to an existing emitter while the offline Firewheel callback is under the allocation guard. This does not cover non-empty reflection snapshots; that separate unsafe native-buffer handoff is documented in `docs/research/spatial-audio-alternatives.md` and keeps spatial task 4.8 open.
- [x] 1.5 Strictly validate the OpenSpec change and document how to launch the game and designer together.
