# Pabi

[![Development](https://img.shields.io/badge/development-work%20in%20progress-red)](https://github.com/github/kirillbobyrev/pabi)
[![Docs](https://docs.rs/pabi/badge.svg)](https://docs.rs/pabi)
[![Depdendencies](https://deps.rs/repo/github/kirillbobyrev/pabi/status.svg)](https://deps.rs/repo/github/kirillbobyrev/pabi)
[![codecov](https://codecov.io/gh/kirillbobyrev/pabi/branch/main/graph/badge.svg)](https://codecov.io/gh/kirillbobyrev/pabi)
[![Lines of Code](https://tokei.rs/b1/github/kirillbobyrev/pabi)](https://github.com/kirillbobyrev/pabi/tree/main/src)

[![Build](https://github.com/kirillbobyrev/pabi/actions/workflows/build.yaml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/build.yaml)
[![Lint](https://github.com/kirillbobyrev/pabi/actions/workflows/lint.yaml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/lint.yaml)
[![Test Suite](https://github.com/kirillbobyrev/pabi/actions/workflows/test.yaml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/test.yaml)
[![Build docs](https://github.com/kirillbobyrev/pabi/actions/workflows/docs.yaml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/docs.yaml)
[![Test Coverage](https://github.com/kirillbobyrev/pabi/actions/workflows/coverage.yaml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/coverage.yaml)

Pabi is a modern hobby chess engine.

## Goals

Pabi is inspired by a number of existing Chess and Go engines. Many of these
amazing and strong engines were started over a decade ago: they are hard to
understand, use and modify.

In contrast, Pabi strives to be __modern__ and __high-quality__ chess engine
focusing on standard chess (as opposed to Fisher Random and other notable
variations).

__Modern__ means that it uses up-to-date [Rust] toolchain, is targeting new
processor architectures, uses latest developments in the domains of programming
tooling and (possibly) Machine Learning.

__High-quality__ means that Pabi will take full advantage of

- Unit testing and integration testing: the majority of the codebase is
  well-tested and the HEAD is in the functional state. Also, advanced
  testing techniques such as [Fuzzing] are used.
- Performance testing and benchmarking: pabi should be __fast__. Each change
  should be tested and confirmed to not cause any performance regressions.
- Continuous quality assurance: GitHub has awesome infrastructure. [GitHub
  Actions] make it possible to test, benchmark and warn on
  regressions/failures. [Dependabot] will make sure the dependencies are
  up-to-date and security issues are resolved in time.
- Documentation: [rustdoc] has awesome features, e.g. warning on
  unudocumented code and testing example code in the documentation.

Pabi strives to be __user-__ and __developer-friendly__.

- Building/downloading it should be straightforward. Anyone willing to try
  it out can either run it locally (possibly in the [Web UI]) or on Lichess
  with stronger hardware ([Lichess deployment]).
- The code is easy to follow and understand. Where appropriate, links to
  [Chess Programming Wiki] are inserted to complement the documentation.
- Development process is easy: the infrastructure is set up in a way that
  minimizes manual labor.
- Contributing is low-effort: there's a [guide for new contributors] and
  [architecture overview].
- Modular and orthogonal architecture makes it easy to isolate changes,
  making their impact obvious in combination with CI and automated checks
  run in each Pull Request.

Pabi strives to provide a __platform__ for exploration. Testing new ideas is
only valuable if the outcome if such tests can be well understood and can be
interpreted. There are plenty of interesting ideas to be tried (e.g. different
evaluation network architecture, different training methods and network
distillation/quantization as means of compression). See [Resources] for a list
of candidates.

Pabi strives to be __strong__. It should participate in [Computer Chess Rating
Lists] (CCRL) and ultimately, if and when it reaches 3000+ ELO strength, in
Chess.com [Computer Chess Championship] (CCCC)[^cccc]. As such, it should be
fine-tuned for the time formats (mainly Blitz and Rapid) that are popular in
these rating lists and designed to perform well against other engines.
Performance and good time-management are crucial for doing well in
time-pressure scenarios.

These goals are indeed very ambitions and only time will tell if they will
be fulfilled.

### Personal motivation

I decided to build a chess engine because of my passion for both chess and
programming. The world of Chess Programming is relatively well-researched at
this point and there is no shortage of strong chess engines. However,
building my own is a great way to

- Challenge myself: can I dive into the area I have little experience in?
  Can I implement a program in a language I am not very proficient in? Can I
  produce high-quality code? Can I build an engine that could beat strong
  human players and potentially other strong chess engines?
- Finish a project: to this day, I never finished a personal projects (it
  just feels way easier at work because of social expectation) and don't
  have anything to showcase my skills. Finally finishing a project (to a
  degree that I would be happy to show it to others) that I could be proud
  of sounds like an exciting opportunity.
- Learn: chess programming has a number of interesting techniques and fun
  algorithms to implement, many of those are challenging (parallel
  execuution, bitboard representation, performance tuning). Practicing and
  sharepning my skills as a developer in a project I am excited about is a
  lot of fun. I own a copy of [Reinforcement Learning: An Introduction]
  (Magnum Opus in the world of Reinforcement Learning from a founding father
  and PhD advisor of AlphaGo creator David Silver) and am learning a lot
  about RL in this context.
- Teach: building a "reference engine" with lots of documentation and
  high-quality Rust code can be beneficial to other developers. It
  encourages others to learn more about Chess Programming and shows a way
  use Rust in performance-critical software. Many resources in Chess
  Programming Wiki and elsewhere are hard to follow without enough
  context/dedication to study, Pabi can illustrate some of the important
  concepts.
- Try Rust: Rust seems to be great for a pre-AlphaGo era engine implementation
  (heuristic-based position evaluation). It is feature-rich, fast, modern and
  convenient programming language for systems programming. In a way, it feels
  like a natrual fit for such projects because it enforces good engineering
  practices into the development process. I would guess that it might be easier
  to fix some of the shortcomings of existing engines and/or prevent them.
  However, the language is still not as widely adopted as C or C++. As the
  result, some of the infrastructure is not as mature yet: Machine Learning
  infrastructure ([Are We Learning Yet?]) is a notable example. Initially, this
  was the reason for my concerns because if I decide to go with the Neural
  Network-based evaluation, lack of infrastructure would make it complicated.
  However, the fashion is using [NNUE] and it can be used even in the absense
  of any ML infrastructure (through hard-coding the network in code, which is
  usually done in C or C++ engines). For more sophisticated techniques, I can
  use [Pytorch Rust bindings] and either split training (e.g. into a Python
  module) and inference (engine real-time evaluation), or implement both in
  Rust (though that would be more challenging). In the end, Rust is an amazing
  language with tons of crates that simplify the development process (I don't
  have to reimplement most basic things like Command-Line arguments parsing etc
  in the fashion of C and C++ projects where pulling a dependency is painful)
  and very strong engines can be implemented even in languages seemingly not
  well-suited for it (e.g. [Ceres]). In the end, this is an interesting test to
  put Rust as a production-ready programming language through!
- Come up with new ideas: I feel like I have some expertise in fields of
  Compilers, Machine Learning and Reinforcement Learning that might allow me
  to come up with new ideas and potentially contribute novel approaches.

## [Milestones]

The project is currently in pre-0.1.0 stage. Therefore, planning is hard and
might significantly change depending on my availability and interest in the
project.

- [0.1.0] should be the skeleton of a chess engine. Most important features are
  implemented or drafted, the performance testing is put in-place. There is a
  Command-Line Interface for interacting with the engine, debugging it and
  trying it out.
- [1.0.0]: Pabi is a conventional pre [AlphaGo] era chess engine. It uses
  heuristtics + MCTS for evaluattion and search. It can be played locally or
  online, although its strengh is not State-of-The-Art yet.
- [2.0.0] is a possible ulttimate goal. Reinforcement Learning is used to
  increase strength and possibly compete with strong human players and other
  engines. I don't know if I will have enough time and dedication to reach this
  stage, but it would be great to get there at some point.

[Rust]: https://www.rust-lang.org/
[Fuzzing]: https://en.wikipedia.org/wiki/Fuzzing
[GitHub Actions]: https://github.com/features/actions
[Dependabot]: https://github.com/dependabot
[rustdoc]: https://doc.rust-lang.org/rustdoc
[Web UI]: https://github.com/kirillbobyrev/pabi/issues/14
[Lichess deployment]: https://github.com/kirillbobyrev/pabi/issues/14
[Chess Programming Wiki]: https://www.chessprogramming.org/Main_Page
[guide for new contributors]: https://github.com/kirillbobyrev/pabi/issues/15
[architecture overview]: https://github.com/kirillbobyrev/pabi/issues/4
[Resources]: https://github.com/kirillbobyrev/pabi/wiki/Resources
[Computer Chess Rating Lists]: http://ccrl.chessdom.com/
[Computer Chess Championship]: https://www.chess.com/computer-chess-championship
[Reinforcement Learning: An Introduction]: http://incompleteideas.net/book/the-book.html
[AlphaGo]: https://en.wikipedia.org/wiki/AlphaGo
[Are We Learning Yet?]: https://www.arewelearningyet.com/
[NNUE]: https://www.chessprogramming.org/NNUE
[Pytorch Rust bindings]: https://github.com/LaurentMazare/tch-rs
[Ceres]: https://github.com/dje-dev/Ceres
[Milestones]: https://github.com/kirillbobyrev/pabi/milestones
[0.1.0]: https://github.com/kirillbobyrev/pabi/milestone/1
[1.0.0]: https://github.com/kirillbobyrev/pabi/milestone/2
[2.0.0]: https://github.com/kirillbobyrev/pabi/milestone/3

[^cccc]:
    More technical and format details on CCCC:
    <https://www.chess.com/news/view/announcing-the-new-computer-chess-championship>
