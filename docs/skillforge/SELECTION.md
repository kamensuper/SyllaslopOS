# Selection and maintenance

Four project skills cover current work. Their scope follows the code and the
established QEMU paths at source baseline
[`d58adb7f3dfe109c327bba17f6d49f22058a4a53`](https://github.com/kamensuper/SyllaslopOS/tree/d58adb7f3dfe109c327bba17f6d49f22058a4a53).

| Installed in the project | Decision | When to reconsider |
| --- | --- | --- |
| `syllaslop-boot` | Keep machine entry, linker and loader ownership together. The upstream x86 and GNU ld examples need the actual GRUB ISO and physical placement. | Split out early memory guidance after mappings or frame ownership have stable entry points. |
| `syllaslop-rust` | Merge bounds, ABI, unsafe and memory ownership. Host-based Rust examples are inputs to review. The actual freestanding target and ownership contracts remain decisive. | Add an interrupts or userspace skill after real entry code and tests appear. |
| `syllaslop-console` | Keep current keyboard, serial, framebuffer and UI paths together. Terminal UI libraries are design references for damage and input rather than guest dependencies. | Split graphics only if other guest subsystems start using the renderer. |
| `syllaslop-verify` | Adapt first divergence and fresh evidence from Superpowers. Name the expectation, failure fixture and artifact. Preserve cleanup and bounded probes. | Add specific tracing or property-testing recipes after repeat use. |

A separate Bible skill would duplicate the route table. Filesystem, allocator,
interrupt and ring-3 skills would claim stable interfaces before the code
exists. Their references are indexed and can be used as architectural inputs.

Superpowers is useful in pieces. Keep minimal failure reproduction, red-green
tests where the seam exists, and fresh acceptance evidence. Use a short
design or plan when invariants are unsettled. Distinguish design exploration
from predictable implementation. Compare review claims to observable
behavior. Avoid automatic workflow chains, per-task delegation, blanket review,
or compulsory brainstorming for a local repair. The independent review pilot
found a real false pass, which supports reviewing tricky success predicates
even if the original change is only one line.

## Library, tutorial and specification references

| Reference | Job |
| --- | --- |
| [Agent Skills specification](https://agentskills.io/specification), [agentskills repository](https://github.com/agentskills/agentskills/tree/83021e2be457a9b6679148719c1f6074f0ad975e), [Codex discovery](https://developers.openai.com/codex/skills) | Spec and client documentation for metadata, directories and progressive disclosure. The reference validator is tooling. |
| [Philipp Oppermann's Writing an OS in Rust](https://os.phil-opp.com/) and [rust-osdev](https://github.com/rust-osdev) | Tutorial plus maintained org libraries, including x86-64 abstractions and Multiboot parsing. Boot approach and crate build differ from this project. |
| [OSDev Wiki](https://wiki.osdev.org/), [Intel SDM](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html), [GNU Multiboot2](https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html), [UEFI Forum specifications](https://uefi.org/specifications) | Use community articles for orientation. Check ABI, page table, entry conditions and device claims against primary specifications. |
| [Rust bare-metal target](https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-none.html) and [QEMU GDB](https://www.qemu.org/docs/master/system/gdb.html) | Verify actual compiler target and the emulator's debug mechanism. |
| [Ratatui](https://github.com/ratatui/ratatui) | Host TUI library. Rendering and terminal interaction provide analogies for update boundaries and visual tests. |
| [skills.sh](https://skills.sh/) | Discovery index. Use candidates' source, license and executable behavior for decisions. |

[ROUTING.md](ROUTING.md) covers areas with no credible installable project
skill, including FAT32/VFS, application SDK, ring-3 isolation, PCI, ACPI,
DMA, storage, networking, and visual accessibility. It pairs current
project entry points with Bible volumes and design checks.

## Provenance and scope

Project skill bodies are original guidance under the repo's terms. External
files, hooks and helpers were inspected as data and remain external. Avoid
installing dormant dependencies or running remote installer instructions.
When evaluating a future candidate, pin the revision, inspect the exact
skill and scripts that it can call, and check writes, network, secrets,
dependency installation and license compatibility before execution.
Treat repository claims as hypotheses until checked against the source.

Use `tools/skillforge/check.py` with Python 3. The optional Codex discovery
check uses local stdio and a temporary profile. Benchmark fixtures need
Python 3, Rust and a POSIX host with `/proc`. QEMU verification uses the
repository's existing Make dependencies. No global configuration changes
are necessary.
