The Philosophical Developer
Chapter 3

Building with a tether.

I had this thought a while back. When an AI writes code, it guesses. It guesses function names, return types, what a library exports. And sometimes it guesses wrong. The code compiles but the logic is built on thin air.

I wondered: what if we could tether the AI to something real during the writing process? Something that catches the bad guess before it becomes a test failure or, worse, a bug in production.

That something is LSP. The Language Server Protocol. Every editor uses it. When you type in VS Code and red squiggles appear under a wrong type, that is LSP at work. My thought was: hook the AI into the same loop. Let it see the red squiggles as it writes, not after.

I asked my assistant to build a small Rust project using this idea. A CLI argument parser from scratch. No external libraries. Just pure Rust and the LSP tether.

The rule was simple. Write a test. Check LSP. Run the test. See it fail. Write the code. Check LSP again. Run the test. See it pass. Clean up. Repeat.

Ten cycles. One behavior at a time. First an empty command. Then a positional argument. Then multiple arguments. Then flags, options, subcommands, help text, error handling.

Every time the AI wrote something wrong, LSP caught it before the compiler. A recursive type without proper wrapping. A missing trait derive. A method call on the wrong type. Small things, but they break the flow. Catching them at write speed instead of compile speed kept the loop tight.

The result was 11 tests, 86 percent coverage, and a working argument parser called mini-clap. The code is on GitHub at dark5un/mini-clap.

What surprised me was not the code quality. It was the confidence. When you watch each test fail first, then pass, you know exactly what each piece of code does. There is no mystery. There is no trust. There is verification.

The LSP tether is not about catching every error. The compiler does that anyway. It is about catching them sooner. When you are going through ten cycles, saving one compile per cycle adds up. And when you are an AI writing code, every cycle is a chance to drift. The tether pulls you back.