## My First Emulator (CHIP-8)

![chip8](https://github.com/user-attachments/assets/a36576f4-52a3-49c6-aeed-6ad1094de0d1)

I wanted to get into the world of emulator writing, with the primary goal of writing a GameBoy emulator. All the advice seemed to be suggesting I write a CHIP-8 emulator first, so here we are.

---

It's a very simple implementation with few things worth mentioning, however:

- it implements the quirks of the original system, as set out here: [https://chip8.gulrak.net/](https://chip8.gulrak.net/)
- the "screen" is refreshed every frame (60FPS) so that it's possible to simulate pixel fading to prevent most flickering
- the emulator runs in a separate thread to the window and loops as fast as possible to get exactly 60Hz (this is very inefficient of course, but simplest for this toy project)
- it passes all the tests from Timendus's suite (which were a godsend when making sure everything was implemented correctly): [https://github.com/Timendus/chip8-test-suite](https://github.com/Timendus/chip8-test-suite)
- the sound isn't actually implemented and instead changes the windows title to a 🔊 emoji (which is why it looks like it flickers)

#### Resources
- [Cowgod's Chip-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
- [CHIP-8 Variant Opcode Table](https://chip8.gulrak.net/)
- [Timendus's Chip-8 Test Suite](https://github.com/Timendus/chip8-test-suite)
