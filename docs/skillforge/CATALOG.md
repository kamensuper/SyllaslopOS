# Skill research catalog

Compatibility refers to runnable instructions in the current SyllaslopOS
environment. Partial candidates contain transferable guidance with incompatible
assumptions. An **adapt** verdict means the project uses a pattern in original
guidance. The repository imports no third-party skill or helper.

Bytes are measured from the pinned whole SKILL.md. The approximate token range
uses 3–5 UTF-8 bytes per token and excludes client prompt wrappers. Optional
references are named in dependencies or risk where material. None are loaded
automatically, so the project activation budget for them is zero. Maintenance
dates are each repository's inspected HEAD, not a claim about every file.

## Pins and license checks

| Source | Inspected revision | Last commit date | License evidence |
| --- | --- | --- | --- |
| mohitmishra786/low-level-dev-skills | [18f29518a47f](https://github.com/mohitmishra786/low-level-dev-skills/tree/18f29518a47fda070e6ec1268cb577692fae9a10) | 2026-11-20 | MIT |
| obra/superpowers | [22f0b463e7a5](https://github.com/obra/superpowers/tree/22f0b463e7a5e579904dd6211a520321035526f7) | 2026-11-03 | MIT |
| rovinax/embedded-skills | [022ce31b469b](https://github.com/rovinax/embedded-skills/tree/022ce31b469b1a1d0c1261c2c8d0f3e07b2c0bbc) | 2026-05-24 | MIT |
| actionbook/rust-skills | [5c40d3ad7851](https://github.com/actionbook/rust-skills/tree/5c40d3ad785193231b7d0dbfb8e1eb447e5edd94) | 2026-08-23 | MIT claimed in README, no top-level license seen |
| full-stack-skills/rust-skills | [b8a6af74af28](https://github.com/full-stack-skills/rust-skills/tree/b8a6af74af28ee86b8cbbbc3075cde0b31a5155e) | 2026-09-24 | Apache-2.0 |
| anthropics/skills | [e4bb5d960d87](https://github.com/anthropics/skills/tree/e4bb5d960d87126ff3b3a8af1a104b7db67560ba) | 2026-11-16 | Apache-2.0 for this skill |
| openai/skills | [49f948faa925](https://github.com/openai/skills/tree/49f948faa9258a0c61caceaf225e179651397431) | 2026-06-23 | Apache-2.0 for this skill |

## mohitmishra786/low-level-dev-skills

| Skill and pinned source | OSDev purpose | Compatibility | Quality | Body bytes / est. tokens | Dependencies | Risk | Overlap | Verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [os-dev-scratch](https://github.com/mohitmishra786/low-level-dev-skills/blob/18f29518a47fda070e6ec1268cb577692fae9a10/skills/kernel/os-dev-scratch/SKILL.md) | x86 entry, PIC and serial bring-up | partial | Good topology overview | 7,036 / 1,407–2,346 | GNU cross tools, QEMU | C, mixed BIOS paths and direct ELF -kernel differ from the GRUB ISO | boot, rust | adapt |
| [assembly-x86](https://github.com/mohitmishra786/low-level-dev-skills/blob/18f29518a47fda070e6ec1268cb577692fae9a10/skills/low-level-programming/assembly-x86/SKILL.md) | Disassembly and SysV call inspection | partial | Useful call and register checklist | 5,905 / 1,181–1,969 | objdump, GDB, GCC examples | Hosted C examples need freestanding ABI and red-zone context | boot, rust | adapt |
| [linker-scripts](https://github.com/mohitmishra786/low-level-dev-skills/blob/18f29518a47fda070e6ec1268cb577692fae9a10/skills/embedded/linker-scripts/SKILL.md) | Distinguish VMA, LMA and section boundaries | partial | Useful GNU ld examples | 9,448 / 1,889–3,150 | GNU ld, readelf, ARM example | Flash and startup copy recipes are target-specific | boot | adapt |
| [bootloaders-embedded](https://github.com/mohitmishra786/low-level-dev-skills/blob/18f29518a47fda070e6ec1268cb577692fae9a10/skills/baremetal/bootloaders-embedded/SKILL.md) | Application handoff ownership | MCU only | Clear source-to-handoff checklist | 4,310 / 862–1,437 | Cortex-M, DFU tools | VTOR and flashing are irrelevant to x86_64 BIOS | boot | reject |
| [virtual-memory-paging-and-tlb](https://github.com/mohitmishra786/low-level-dev-skills/blob/18f29518a47fda070e6ec1268cb577692fae9a10/skills/computer-architecture/virtual-memory-paging-and-tlb/SKILL.md) | Review PTE, fault and TLB assumptions | partial | Useful concepts, weak project checks | 2,965 / 593–989 | Linux /proc, host tools | Linux counters cannot establish guest TLB behavior | rust, Bible V08 | adapt |
| [kernel-memory-management](https://github.com/mohitmishra786/low-level-dev-skills/blob/18f29518a47fda070e6ec1268cb577692fae9a10/skills/kernel-dev/kernel-memory-management/SKILL.md) | Allocator tradeoffs and terminology | Linux only | Sound orientation for Linux allocation | 3,451 / 690–1,151 | Linux kernel APIs | SLUB, zones and vmalloc are not available here | Bible V07–09 | reject |
| [kernel-concurrency](https://github.com/mohitmishra786/low-level-dev-skills/blob/18f29518a47fda070e6ec1268cb577692fae9a10/skills/kernel-dev/kernel-concurrency/SKILL.md) | Locking and interrupt context | partial | Good concept prompts | 3,326 / 665–1,109 | Linux locks, RCU, PREEMPT_RT | Kernel APIs and preemption assumptions differ | rust, Bible V11 | defer |
| [linux-kernel-architecture](https://github.com/mohitmishra786/low-level-dev-skills/blob/18f29518a47fda070e6ec1268cb577692fae9a10/skills/kernel-dev/linux-kernel-architecture/SKILL.md) | Navigate a kernel source tree | Linux only | Useful Linux source map | 3,118 / 623–1,040 | Linux source tree | Initcalls and subsystem paths misroute SLC | Bible routing | reject |
| [rust-for-linux](https://github.com/mohitmishra786/low-level-dev-skills/blob/18f29518a47fda070e6ec1268cb577692fae9a10/skills/kernel-dev/rust-for-linux/SKILL.md) | Kernel Rust integration | Linux only | Detailed examples for that target | 12,351 / 2,470–4,117 | Linux kernel crate, bindgen, KUnit | Install commands, Linux APIs and example license declarations | rust | reject |
| [device-drivers](https://github.com/mohitmishra786/low-level-dev-skills/blob/18f29518a47fda070e6ec1268cb577692fae9a10/skills/kernel/device-drivers/SKILL.md) | Driver lifetime and DMA concepts | partial | Good lifecycle vocabulary | 7,510 / 1,502–2,504 | Linux platform APIs, udev | Linux coherent DMA APIs are not guest hardware contracts | rust, Bible V15 | defer |
| [kernel-testing](https://github.com/mohitmishra786/low-level-dev-skills/blob/18f29518a47fda070e6ec1268cb577692fae9a10/skills/kernel/kernel-testing/SKILL.md) | Guest and host test separation | Linux only | Specific Linux test recipes | 5,852 / 1,170–1,951 | KUnit, kselftest, Linux test tooling | Wrong harness, dependencies and architecture for SLC | verify | reject |
| [qemu-for-kernel-development](https://github.com/mohitmishra786/low-level-dev-skills/blob/18f29518a47fda070e6ec1268cb577692fae9a10/skills/kernel-dev/qemu-for-kernel-development/SKILL.md) | Emulated kernel debugging | partial | Useful serial and paused start concepts | 3,241 / 648–1,081 | QEMU, Linux images, Buildroot | KVM and Linux kernel boot examples differ from TCG and GRUB | boot, verify | adapt |
| [gdb](https://github.com/mohitmishra786/low-level-dev-skills/blob/18f29518a47fda070e6ec1268cb577692fae9a10/skills/debuggers/gdb/SKILL.md) | Early kernel symbol and register debugging | partial | Good breakpoint and watchpoint toolbox | 6,331 / 1,266–2,111 | GDB, debug symbols | Hosted gdbserver and libc assumptions do not apply to bare metal | verify | adapt |
| [kernel-debugging-advanced](https://github.com/mohitmishra786/low-level-dev-skills/blob/18f29518a47fda070e6ec1268cb577692fae9a10/skills/kernel-dev/kernel-debugging-advanced/SKILL.md) | Trace and profile running code | Linux only | Clear trace tooling | 3,433 / 686–1,145 | ftrace, perf, kgdb | Missing Linux hooks and guest instrumentation | verify | reject |
| [rust-profiling](https://github.com/mohitmishra786/low-level-dev-skills/blob/18f29518a47fda070e6ec1268cb577692fae9a10/skills/rust/rust-profiling/SKILL.md) | Profile host-testable Rust and assess sizes | partial | Concrete measurement ideas | 6,637 / 1,327–2,213 | perf, Cargo extensions, Criterion | Host profiler results do not measure QEMU input latency | verify, Bible V24 | defer |
| [rust-unsafe](https://github.com/mohitmishra786/low-level-dev-skills/blob/18f29518a47fda070e6ec1268cb577692fae9a10/skills/rust/rust-unsafe/SKILL.md) | Audit raw pointer and ABI contracts | partial | Solid invariants and review checklist | 7,771 / 1,554–2,591 | Rust, hosted examples | std examples, proof must include device-specific mapping and lifetime | rust | adapt |
| [rust-no-std](https://github.com/mohitmishra786/low-level-dev-skills/blob/18f29518a47fda070e6ec1268cb577692fae9a10/skills/rust/rust-no-std/SKILL.md) | Host tests beside freestanding build | partial | Useful separation of core, alloc and std | 7,518 / 1,503–2,506 | Rust, Cargo, thumb example | Manifest and allocator examples do not describe the Make build | rust | adapt |
| [rust-sanitizers-miri](https://github.com/mohitmishra786/low-level-dev-skills/blob/18f29518a47fda070e6ec1268cb577692fae9a10/skills/rust/rust-sanitizers-miri/SKILL.md) | Validate pure unsafe logic on a supported host | partial | Useful boundary checks and limits | 6,425 / 1,285–2,142 | nightly, Cargo, Miri, sanitizers | Host diagnostics cannot verify MMIO or CPU state | rust, verify | defer |
| [fuzzing](https://github.com/mohitmishra786/low-level-dev-skills/blob/18f29518a47fda070e6ec1268cb577692fae9a10/skills/runtimes/fuzzing/SKILL.md) | Exercise parsers against malformed inputs | partial | Good corpus and failure-replay guidance | 9,023 / 1,804–3,008 | C harness, libFuzzer, AFL++ | Toolchain and examples do not carry directly to freestanding Rust | verify | defer |
| [formal-verification](https://github.com/mohitmishra786/low-level-dev-skills/blob/18f29518a47fda070e6ec1268cb577692fae9a10/skills/runtimes/formal-verification/SKILL.md) | Model checked arithmetic and bounded algorithms | partial | Concrete proof-harness examples | 12,496 / 2,499–4,166 | Kani, CBMC or other verifiers | Installer downloads and model assumptions need review | verify | defer |
| [embedded-rust](https://github.com/mohitmishra786/low-level-dev-skills/blob/18f29518a47fda070e6ec1268cb577692fae9a10/skills/embedded/embedded-rust/SKILL.md) | Firmware panic and peripheral testing | MCU only | Concrete MCU workflows | 6,383 / 1,276–2,128 | probe-rs, cargo-embed, RTIC | Includes remote installer piped to shell, flash commands change devices | rust | reject |

## obra/superpowers

| Skill and pinned source | OSDev purpose | Compatibility | Quality | Body bytes / est. tokens | Dependencies | Risk | Overlap | Verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [systematic-debugging](https://github.com/obra/superpowers/blob/22f0b463e7a5e579904dd6211a520321035526f7/skills/systematic-debugging/SKILL.md) | Reproduce the first divergence | compatible | Strong red-first and root-cause separation | 9,530 / 1,906–3,177 | Optional shell and TS resources | Bundled test-loop script and broad mandatory wording | verify | adapt |
| [verification-before-completion](https://github.com/obra/superpowers/blob/22f0b463e7a5e579904dd6211a520321035526f7/skills/verification-before-completion/SKILL.md) | Check fresh evidence before success claims | compatible | Directly useful completion gate | 3,646 / 729–1,216 | Project test commands | A green run still needs the right predicate and scope | verify | adapt |
| [test-driven-development](https://github.com/obra/superpowers/blob/22f0b463e7a5e579904dd6211a520321035526f7/skills/test-driven-development/SKILL.md) | Protect parser and oracle regressions | partial | Useful red-green seam | 9,578 / 1,915–3,193 | Project test runners | Universal no-code-before-red gate can overfit early hardware bring-up | rust, verify | adapt |
| [writing-plans](https://github.com/obra/superpowers/blob/22f0b463e7a5e579904dd6211a520321035526f7/skills/writing-plans/SKILL.md) | Set contracts for architectural changes | compatible | Good scope and dependency questions | 10,335 / 2,067–3,445 | Markdown and linked workflows | Large global plan and downstream workflow requirement add overhead | tier policy | adapt |
| [requesting-code-review](https://github.com/obra/superpowers/blob/22f0b463e7a5e579904dd6211a520321035526f7/skills/requesting-code-review/SKILL.md) | Provide review contract, diff and evidence | partial | Handoff template is applicable | 2,974 / 594–992 | Subagent adapter if used | Per-task review adds overhead and requires agent support | critical tier | adapt |
| [receiving-code-review](https://github.com/obra/superpowers/blob/22f0b463e7a5e579904dd6211a520321035526f7/skills/receiving-code-review/SKILL.md) | Verify review claims technically | compatible | Useful disagreement and validation guidance | 6,203 / 1,240–2,068 | Project tools | Most guidance is generic and duplicates normal review discipline | verify | adapt |
| [executing-plans](https://github.com/obra/superpowers/blob/22f0b463e7a5e579904dd6211a520321035526f7/skills/executing-plans/SKILL.md) | Inline execution across staged work | partial | Useful checkpoint discipline | 18,364 / 3,672–6,122 | Worktrees, follow-on workflows | Large instruction body and handoff assumptions for routine repairs | balanced tier | defer |
| [subagent-driven-development](https://github.com/obra/superpowers/blob/22f0b463e7a5e579904dd6211a520321035526f7/skills/subagent-driven-development/SKILL.md) | Independent implementation and review on high-risk changes | partial | Useful context isolation and review split | 19,740 / 3,948–6,580 | Subagents, worktrees, reviewer prompts | Fresh contexts, coordination and final review multiply cost | critical tier | defer |
| [brainstorming](https://github.com/obra/superpowers/blob/22f0b463e7a5e579904dd6211a520321035526f7/skills/brainstorming/SKILL.md) | Explore architecture alternatives before commitment | partial | Discovery questions help ambiguous design | 15,190 / 3,038–5,064 | Optional visual and builder-check workflows | Broad activation and approval gating expensive for deterministic work | critical tier | defer |

## rovinax/embedded-skills

| Skill and pinned source | OSDev purpose | Compatibility | Quality | Body bytes / est. tokens | Dependencies | Risk | Overlap | Verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [embedded-isr-design](https://github.com/rovinax/embedded-skills/blob/022ce31b469b1a1d0c1261c2c8d0f3e07b2c0bbc/skills/embedded-isr-design/SKILL.md) | Review IRQ handler latency and ownership | partial | Deferred-work distinction is useful | 12,396 / 2,479–4,132 | MCU and RTOS examples | Cortex-M register state and ISR timing budget do not transfer | rust, Bible V10 | defer |
| [embedded-driver-design](https://github.com/rovinax/embedded-skills/blob/022ce31b469b1a1d0c1261c2c8d0f3e07b2c0bbc/skills/embedded-driver-design/SKILL.md) | Separate bus, device and app state | partial | Concrete lifetime and driver layering | 9,896 / 1,979–3,299 | C HAL or MCU platform | Prescribed directory layout and C rules differ | rust, Bible V15 | defer |
| [embedded-rtos-design](https://github.com/rovinax/embedded-skills/blob/022ce31b469b1a1d0c1261c2c8d0f3e07b2c0bbc/skills/embedded-rtos-design/SKILL.md) | Reason about priority and ISR deferral | partial | Useful scheduling vocabulary | 9,790 / 1,958–3,264 | FreeRTOS, Zephyr and other RTOS APIs | Existing RTOS statistics and priorities assume a scheduler | Bible V11 | defer |

## actionbook/rust-skills

| Skill and pinned source | OSDev purpose | Compatibility | Quality | Body bytes / est. tokens | Dependencies | Risk | Overlap | Verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [unsafe-checker](https://github.com/actionbook/rust-skills/blob/5c40d3ad785193231b7d0dbfb8e1eb447e5edd94/skills/unsafe-checker/SKILL.md) | Audit unsafe blocks and layout | partial | Short and actionable checklist | 2,490 / 498–830 | Optional FFI ecosystem tools | Hosted C string examples and broad triggers | rust | adapt |
| [domain-embedded](https://github.com/actionbook/rust-skills/blob/5c40d3ad785193231b7d0dbfb8e1eb447e5edd94/skills/domain-embedded/SKILL.md) | Split device startup, drivers and runtime | partial | Useful domain questions | 4,509 / 901–1,503 | MCU HAL, Embassy or RTIC | Auto-injected context and platform stack assume MCU ecosystems | rust | defer |
| [m07-concurrency](https://github.com/actionbook/rust-skills/blob/5c40d3ad785193231b7d0dbfb8e1eb447e5edd94/skills/m07-concurrency/SKILL.md) | Review ownership and atomic sharing | partial | Practical Send and Sync prompts | 6,341 / 1,268–2,114 | Hosted threads and Tokio examples | Cross-skill trace-up and hosted primitives are poor defaults | rust, Bible V11 | defer |
| [domain-cli](https://github.com/actionbook/rust-skills/blob/5c40d3ad785193231b7d0dbfb8e1eb447e5edd94/skills/domain-cli/SKILL.md) | Search and CLI usability patterns | partial | Useful test and help ideas | 3,990 / 798–1,330 | clap, crossterm, ratatui | Host terminal, argument and config APIs do not exist in the framebuffer guest | console | defer |

## full-stack-skills/rust-skills

| Skill and pinned source | OSDev purpose | Compatibility | Quality | Body bytes / est. tokens | Dependencies | Risk | Overlap | Verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [rust-unsafe-ffi](https://github.com/full-stack-skills/rust-skills/blob/b8a6af74af28ee86b8cbbbc3075cde0b31a5155e/skills/rust-unsafe-ffi/SKILL.md) | Review aliasing, layout and ABI | partial | Strong audit checklist and a host/platform distinction | 7,104 / 1,420–2,368 | Cargo, Miri, optional C tools | Example tooling and unwinding support differ | rust | adapt |
| [rust-embedded](https://github.com/full-stack-skills/rust-skills/blob/b8a6af74af28ee86b8cbbbc3075cde0b31a5155e/skills/rust-embedded/SKILL.md) | Inspect DMA, ownership and real hardware evidence | partial | Good test-scope discipline | 9,113 / 1,822–3,038 | HAL, Cargo, flashing tools | Board and flashed hardware assumptions differ | rust, verify | defer |
| [rust-concurrency](https://github.com/full-stack-skills/rust-skills/blob/b8a6af74af28ee86b8cbbbc3075cde0b31a5155e/skills/rust-concurrency/SKILL.md) | Reason about ordering, invariants and backpressure | partial | Useful concurrency boundaries | 11,481 / 2,296–3,827 | Cargo, Loom and hosted runtime examples | Tokio/Rayon and OS threads are unavailable | rust, Bible V11 | defer |

## anthropics/skills

| Skill and pinned source | OSDev purpose | Compatibility | Quality | Body bytes / est. tokens | Dependencies | Risk | Overlap | Verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [skill-creator](https://github.com/anthropics/skills/blob/e4bb5d960d87126ff3b3a8af1a104b7db67560ba/skills/skill-creator/SKILL.md) | Design fresh-thread skill comparisons | partial | Useful evaluation structure and selective metadata loading | 33,168 / 6,633–11,056 | Python eval runner and platform integrations | Long body, token telemetry assumptions and optional model graders | this benchmark | adapt |

## openai/skills

| Skill and pinned source | OSDev purpose | Compatibility | Quality | Body bytes / est. tokens | Dependencies | Risk | Overlap | Verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [skill-creator](https://github.com/openai/skills/blob/49f948faa9258a0c61caceaf225e179651397431/skills/.system/skill-creator/SKILL.md) | Validate skill naming and structure | partial | Helpful progressive disclosure and directory layout | 18,664 / 3,732–6,222 | Python helpers and YAML tooling | User-level installation procedure irrelevant to this repository | spec and project linter | adapt |
