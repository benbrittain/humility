
# Humility

Humility is the debugger for
<a href="https://github.com/oxidecomputer/hubris">Hubris</a>.

## Guiding principles

### Production disposition

Hubris is the artifact that actually runs, and it must be allowed to be
optimized in terms of size, space, and run-time; in as much as contortions are
required for debuggability, they should be borne by Humility, not Hubris.  As a
concrete example of this, Humility operates exclusively on `release` builds of
Hubris, relying on (unloaded) DWARF information in the binaries to make sense of
the system.

### Hubris-specific

Humility is Hubris-specific:  it is not trying to be a generic *in situ*
debugger, but rather specifically focused on debugging Hubris-based systems.
Humility is therefore willing to encode Hubris-specific concepts like
packages and tasks.

### Microcontroller-specific

Debuggers must cut through abstractions, which 
often requires knowledge of underlying
implementation detail.  For Humility, this means being willing to take
advantage of microcontroller-specific debug facilities where applicable.
While Hubris may make decisions in the name of greater portability,
Humility will generally make decisions to maximize debuggability, even
where these facilities are highly specific to a particular MCU.

### Pragmatic

Humility seeks to be *useful*, and therefore seeks to offer all manners of
debugging: in situ, postmortem, dynamic instrumentation, static
instrumentation, etc.

## Operation

Humility operates by specifying a subcommand.  There are options that
are Humility-wide (that is, applying to every subcommand), as well as 
options that are specific to particular subcommands.  

### Chip

While some autodetection is in princple possible, Humility regrettably
needs to be made aware of the specifics of the target chip.
Supported chips include:

- `STM32F407VGTx` (default): STM32F407 as found on the reference Discovery board
- `LPC55S69JBD100`: LPC55S69 as found on the LPCXpresso55S69

The target chip can be specified via the `-c` option or the `HUMILITY_CHIP`
environment variable.

### Debugger

Humility needs to have some way of extracting information and/or controlling
the microcontroller running Hubris.  This is done through some variant of a
*debug probe*, a separate microcontroller that speaks to debug-specific
functionality on the target microcontroller.  (For details of the mechanics
of these probes on ARM parts, see <a
href="https://65.rfd.oxide.computer">RFD 65</a>.)

For most evaluation boards, this debug probe is available on the board, and
is connected to a host via USB, e.g.  ST's STLink/V2 on the STM32F407
Discovery or the LPC-Link2 present on the LPCXpresso55S69.  (When we deploy
Hubris on our own board, we will debug via either JTAG or a SWD header.)
While Humility allows for direct attachment to an on-chip debugger, doing so
precludes other connections (from, for example, OpenOCD), making it to
disruptive to development workflows. To allow for easier development
workflows, Humility also has the option to attach via OpenOCD.

The debug probe to use is specified to Humility via
the `-d` option (long form `--debugger`), which can have the following values:

- `auto` (default): Automatically determine how to attach to the 
  microcontroller.

- `ocd`: Attach via OpenOCD, which is presumed to have the TCL interface
  available on localhost on port 6666 (its default).

- `probe`: Attach directly via USB to a debug probe

### Package

Many Humility commands require the complete Hubris package.  This is 
the directory as created by `packager`, and includes the `map.txt` file,
as well as the full DWARF binaries of the package constituents.  This
package directory is specified via the `-p` option or the `HUMILITY_PACKAGE`
environment variable.

## Commands

### `humility tasks`

`humility tasks` offers a ps-like view of a system, e.g.:

```
% humility -p ~/hubris/target/packager tasks
humility: attached via OpenOCD
ID ADDR     TASK               GEN STATE    
 0 200000e8 jefe                 0 Healthy(InRecv(None))     
 1 20000158 rcc_driver           0 Healthy(InRecv(None))     
 2 200001c8 usart_driver         0 Healthy(Runnable)          <-
 3 20000238 ping                 3 Healthy(InReply(4))
 4 200002a8 pong                 0 Healthy(InRecv(None))     
 5 20000318 idle                 0 Healthy(Runnable)         
```

To see every field in each task, you can use the `-v` flag:

```
% humility -p ~/hubris/target/packager tasks -v
...
 4 200002c8 pong                 0 Healthy(InRecv(None))
          |
          +----> {
                    save: {
                        r4: 0x200023bc,
                        r5: 0x10,
                        r6: 0x1,
                        r7: 0x200023f0,
                        r8: 0x40020c00,
                        r9: 0x2491b8,
                        r10: 0x200023bc,
                        r11: 0x1,
                        psp: 0x20002328,
                        exc_return: 0xffffffed
                    },
                    priority: 0x2,
                    state: Healthy(InRecv(None)),
                    timer: {
                        deadline: Some(0x2491b8),
                        to_post: 0x1
                    },
                    generation: 0x0,
                    region_table: {
                        data_ptr: 0x200000a8 (*const &abi::RegionDesc),
                        length: 0x8
                    },
                    notifications: 0x0,
                    descriptor: 0x80047c4 (&abi::TaskDesc)
                }
...
```

### `humility itm`

### `humility etm`


## Future work

### `humility log`

### `humility core`

### `humility stacks`

### `humility test`
