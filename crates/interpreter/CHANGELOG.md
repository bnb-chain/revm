# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [5.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v4.0.0...revm-interpreter-v5.0.0) - 2024-05-12

### Added
- implement EIP-2935 ([#1354](https://github.com/bluealloy/revm/pull/1354))
- parse opcodes from strings ([#1358](https://github.com/bluealloy/revm/pull/1358))
- *(interpreter)* add helpers for spending all gas ([#1360](https://github.com/bluealloy/revm/pull/1360))
- add helper methods to CallInputs ([#1345](https://github.com/bluealloy/revm/pull/1345))
- *(revm)* make `FrameOrResult` serializable ([#1282](https://github.com/bluealloy/revm/pull/1282))
- add flag to force hashbrown usage ([#1284](https://github.com/bluealloy/revm/pull/1284))
- EOF (Ethereum Object Format) ([#1143](https://github.com/bluealloy/revm/pull/1143))
- *(interpreter)* derive Eq for InterpreterAction ([#1262](https://github.com/bluealloy/revm/pull/1262))
- *(interpreter)* remove SPEC generic from gas calculation functions ([#1243](https://github.com/bluealloy/revm/pull/1243))
- *(interpreter)* test Host object-safety, allow `dyn Host` in instructions ([#1245](https://github.com/bluealloy/revm/pull/1245))

### Fixed
- return the correct error in resize_memory ([#1359](https://github.com/bluealloy/revm/pull/1359))
- correct some stack IO ([#1302](https://github.com/bluealloy/revm/pull/1302))

### Other
- add Trin to used by list ([#1393](https://github.com/bluealloy/revm/pull/1393))
- refactor lints ([#1386](https://github.com/bluealloy/revm/pull/1386))
- remove unused file ([#1379](https://github.com/bluealloy/revm/pull/1379))
- *(interpreter)* branch less in as_usize_or_fail ([#1374](https://github.com/bluealloy/revm/pull/1374))
- re-use num_words in gas::cost_per_word ([#1371](https://github.com/bluealloy/revm/pull/1371))
- *(interpreter)* rewrite gas accounting for memory expansion ([#1361](https://github.com/bluealloy/revm/pull/1361))
- remove bounds check in DUP, SWAP/EXCHANGE ([#1346](https://github.com/bluealloy/revm/pull/1346))
- don't clone bytes in `Bytecode::bytes` ([#1344](https://github.com/bluealloy/revm/pull/1344))
- shrink OpCodeInfo and add more methods ([#1307](https://github.com/bluealloy/revm/pull/1307))
- *(interpreter)* rename some macros ([#1304](https://github.com/bluealloy/revm/pull/1304))
- *(interpreter)* remove EOF branch in CODE{SIZE,COPY} ([#1308](https://github.com/bluealloy/revm/pull/1308))
- fix some warnings ([#1305](https://github.com/bluealloy/revm/pull/1305))
- *(interpreter)* rename wrapping_* opcodes ([#1306](https://github.com/bluealloy/revm/pull/1306))
- Add the modifies_memory macro ([#1270](https://github.com/bluealloy/revm/pull/1270))
- *(interpreter)* use `pop_top!` where possible ([#1267](https://github.com/bluealloy/revm/pull/1267))

## [4.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v3.4.0...revm-interpreter-v4.0.0) - 2024-04-02

### Added
- add tests for shift instructions ([#1254](https://github.com/bluealloy/revm/pull/1254))
- derive serde for OpCode, improve implementations ([#1215](https://github.com/bluealloy/revm/pull/1215))
- *(interpreter)* expose mutable access methods on stack and memory ([#1219](https://github.com/bluealloy/revm/pull/1219))

### Other
- use uint macro & fix various small things ([#1253](https://github.com/bluealloy/revm/pull/1253))
- move div by zero check from smod to i256_mod ([#1248](https://github.com/bluealloy/revm/pull/1248))
- *(interpreter)* unbox contract field ([#1228](https://github.com/bluealloy/revm/pull/1228))
- *(interpreter)* keep track of remaining gas rather than spent ([#1221](https://github.com/bluealloy/revm/pull/1221))
- *(interpreter)* don't run signextend with 31 too ([#1222](https://github.com/bluealloy/revm/pull/1222))

## [3.4.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v3.3.0...revm-interpreter-v3.4.0) - 2024-03-19

### Added
- *(interpreter)* export utility macros ([#1203](https://github.com/bluealloy/revm/pull/1203))
- add convert_boxed and insert_boxed for InstructionTable ([#1194](https://github.com/bluealloy/revm/pull/1194))
- optional nonce check ([#1195](https://github.com/bluealloy/revm/pull/1195))

### Other
- expose functionality for custom EVMs ([#1201](https://github.com/bluealloy/revm/pull/1201))
- Fix typo in readme ([#1185](https://github.com/bluealloy/revm/pull/1185))

## [3.3.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v3.2.0...revm-interpreter-v3.3.0) - 2024-03-08

### Added
- *(interpreter)* OpCode struct constants ([#1173](https://github.com/bluealloy/revm/pull/1173))


## [3.2.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v3.1.0...revm-interpreter-v3.2.0) - 2024-03-08

### Added
- add insert method on instruction table ([#1167](https://github.com/bluealloy/revm/pull/1167))
- use `impl` instead of `dyn` in `GetInspector` ([#1157](https://github.com/bluealloy/revm/pull/1157))

### Other
- *(interpreter)* use already-computed sign in SAR ([#1147](https://github.com/bluealloy/revm/pull/1147))
- *(interpreter)* factor out jump logic ([#1146](https://github.com/bluealloy/revm/pull/1146))
- *(interpreter)* evaluate instruction table constructor at compile time ([#1140](https://github.com/bluealloy/revm/pull/1140))

## [3.1.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v3.0.0...revm-interpreter-v3.1.0) - 2024-02-22

### Added
- bump c-kzg, add portable feature, make it default ([#1106](https://github.com/bluealloy/revm/pull/1106))

### Fixed
- replace tuple in sstore return with struct ([#1115](https://github.com/bluealloy/revm/pull/1115))
- *(db)* Set instruction result at outcome insert ([#1117](https://github.com/bluealloy/revm/pull/1117))

### Other
- adding more test for i256 ([#1090](https://github.com/bluealloy/revm/pull/1090))
- *(refactor)* Propagate fatal error ([#1116](https://github.com/bluealloy/revm/pull/1116))
- clippy cleanup ([#1112](https://github.com/bluealloy/revm/pull/1112))

## [3.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v2.1.0...revm-interpreter-v3.0.0) - 2024-02-17

### Fixed
- rename and pass optimism-default-handler to revm-primitives ([#1098](https://github.com/bluealloy/revm/pull/1098))

### Other
- *(precompile)* use `Bytes` in precompile functions ([#1085](https://github.com/bluealloy/revm/pull/1085))
- Add memory offset ([#1032](https://github.com/bluealloy/revm/pull/1032))
- license date and revm docs ([#1080](https://github.com/bluealloy/revm/pull/1080))

## [2.1.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v2.0.0...revm-interpreter-v2.1.0) - 2024-02-12

### Added
- *(interpreter)* relax `make_boxed_instruction_table::FN` to `FnMut` ([#1076](https://github.com/bluealloy/revm/pull/1076))

## [2.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v1.3.0...revm-interpreter-v2.0.0) - 2024-02-07

Iterpreter will not be called in recursive calls but would return Action ( CALL/CREATE) that will be executed by the main loop.

### Added
- tweeks for v4.0 revm release ([#1048](https://github.com/bluealloy/revm/pull/1048))
- add `BytecodeLocked::original_bytecode` ([#1037](https://github.com/bluealloy/revm/pull/1037))
- *(op)* Ecotone hardfork ([#1009](https://github.com/bluealloy/revm/pull/1009))
- EvmBuilder and External Contexts ([#888](https://github.com/bluealloy/revm/pull/888))
- add asm-keccak feature ([#972](https://github.com/bluealloy/revm/pull/972))
- add some conversions to InstructionResult ([#910](https://github.com/bluealloy/revm/pull/910))
- implement Default for InstructionResult ([#878](https://github.com/bluealloy/revm/pull/878))
- `Canyon` hardfork behind `optimism` feature flag ([#871](https://github.com/bluealloy/revm/pull/871))
- Loop call stack ([#851](https://github.com/bluealloy/revm/pull/851))
- *(cfg)* optionally disable beneficiary reward ([#834](https://github.com/bluealloy/revm/pull/834))
- *(interpreter)* add more helper methods to memory ([#794](https://github.com/bluealloy/revm/pull/794))
- derive more traits ([#745](https://github.com/bluealloy/revm/pull/745))
- add methods to `CreateInput` for calculating created address ([#793](https://github.com/bluealloy/revm/pull/793))

### Fixed
- *(Interpreter)* is_revert should call is_revert ([#1007](https://github.com/bluealloy/revm/pull/1007))
- cast overflow in 32-bits OS ([#978](https://github.com/bluealloy/revm/pull/978))
- dont calculate initcode keccak on CREATE ([#969](https://github.com/bluealloy/revm/pull/969))
- *(ci)* Workflow Touchups ([#901](https://github.com/bluealloy/revm/pull/901))
- safer stack ([#879](https://github.com/bluealloy/revm/pull/879))
- *(interpreter)* Stack `push_slice` fix and dup with pointers ([#837](https://github.com/bluealloy/revm/pull/837))

### Other
- helper functions around Env ([#1057](https://github.com/bluealloy/revm/pull/1057))
- *(Execution)* Granular handles create/call,call_return,insert_call_outcome ([#1024](https://github.com/bluealloy/revm/pull/1024))
- *(Interpreter)* Split calls to separate functions ([#1005](https://github.com/bluealloy/revm/pull/1005))
- expose InstructionResult getters in Interpreter result ([#1002](https://github.com/bluealloy/revm/pull/1002))
- *(Inspector)* add CallOutcome to call/call_end ([#985](https://github.com/bluealloy/revm/pull/985))
- fix serde std flags for no-std build ([#987](https://github.com/bluealloy/revm/pull/987))
- *(Inspector)* Add CreateOutcome in create/create_end return ([#980](https://github.com/bluealloy/revm/pull/980))
- *(log)* use alloy_primitives::Log ([#975](https://github.com/bluealloy/revm/pull/975))
- enhance readability ([#968](https://github.com/bluealloy/revm/pull/968))
- *(interpreter)* refactor sstore_cost ([#974](https://github.com/bluealloy/revm/pull/974))
- *(interpreter)* improve enum naming ([#962](https://github.com/bluealloy/revm/pull/962))
- *(interpreter)* consistency in all_results_are_covered() ([#961](https://github.com/bluealloy/revm/pull/961))
- *(interpreter)* local return_error! macro ([#956](https://github.com/bluealloy/revm/pull/956))
- *(interpreter)* simplify the logic of calc.new_cost() ([#939](https://github.com/bluealloy/revm/pull/939))
- *(interpreter)* fix the name of the macro referenced by record_memory() ([#926](https://github.com/bluealloy/revm/pull/926))
- *(interpreter)* conditionally enable `optional_beneficiary_reward` ([#925](https://github.com/bluealloy/revm/pull/925))
- fix case for CreateInitCodeSizeLimit error ([#896](https://github.com/bluealloy/revm/pull/896))
- simplify use statements ([#864](https://github.com/bluealloy/revm/pull/864))
- *(interpreter)* use the constants from primitives ([#861](https://github.com/bluealloy/revm/pull/861))
- review safety comments ([#811](https://github.com/bluealloy/revm/pull/811))
- rewrite `Stack::push_slice` to allow arbitrary lengths ([#812](https://github.com/bluealloy/revm/pull/812))
- make context memory pub ([#831](https://github.com/bluealloy/revm/pull/831))
- refactor main return to handle ([#808](https://github.com/bluealloy/revm/pull/808))
- *(SharedMemory)* small refactor; tests ([#806](https://github.com/bluealloy/revm/pull/806))
- use `array::from_fn` in `make_instruction_table` ([#809](https://github.com/bluealloy/revm/pull/809))
- make memory-limit private ([#796](https://github.com/bluealloy/revm/pull/796))
- Instruction table ([#759](https://github.com/bluealloy/revm/pull/759))
- Shared memory between calls ([#673](https://github.com/bluealloy/revm/pull/673))
- Fix typos ([#790](https://github.com/bluealloy/revm/pull/790))
- document everything, dedup existing docs ([#741](https://github.com/bluealloy/revm/pull/741))

# v1.3.0
date 02.10.2023

Migration to alloy primitive types.

Full git log:
* af4146a - feat: Alloy primitives (#724) (15 hours ago) <evalir>
* 1f86e45 - chore(deps): bump proptest from 1.2.0 to 1.3.1 (#763) (22 hours ago) <dependabot[bot]>

# v1.2.0
date: 28.09.2023

Summary:
* Cancun support:
  * EIP-7516: BLOBBASEFEE opcode
  * EIP-4844: Shard Blob Transactions
  * EIP-1153: Transient storage opcodes
  * EIP-5656: MCOPY - Memory copying instruction
* Rename `SHA3` to `KECCAK256`, this can potentially break some tracers.
* Refactor opcodes and Interpreter dispatch loop. Better performance.
* optimize stack usage for recursive `call` and `create` programs.
    This brings down the native stack usage as calls are in recursion.

Full git log:
* f79d0e1 - feat: Optimism execution changes (#682) (16 hours ago) <clabby>
* d03dfcb - Improve wording and fix typos (#749) (25 hours ago) <Paul Razvan Berg>
* 2c556c0 - refactor: say "warm" instead of "hot" (#754) (25 hours ago) <Paul Razvan Berg>
* 8206193 - feat: add "kzg" as a separate feature (#746) (2 hours ago) <DaniPopes>
* 516f62c - perf(interpreter): remove dynamic dispatch from all instructions (#739) (5 days ago) <DaniPopes>
* 26af13e - EIP-7516: BLOBBASEFEE opcode (#721) (5 days ago) <rakita>
* 36e71fc - fix: dont override instruction result (#736) (6 days ago) <rakita>
* d926728 - perf: refactor interpreter internals and cleanup (#582) (6 days ago) <DaniPopes>
* fa13fea - feat: implement EIP-4844 (#668) (11 days ago) <DaniPopes>
* 190f90e - Never inline the prepare functions (#712) (2 weeks ago) <Valentin Mihov>
* 7eacc3a - chore: implement `Default` for other databases (#691) (3 weeks ago) <DaniPopes>
* 616cc7e - chore(cfg): convert chain_id from u256 to u64 (#693) (3 weeks ago) <Lorenzo Feroleto>
* a95a298 - chore: accept byte slice as input (#700) (3 weeks ago) <Matthias Seitz>
* f6c9c7f - chore: deprecate `RefDBWrapper` (#696) (3 weeks ago) <DaniPopes>
* f2929ad - chore(deps): bump proptest-derive from 0.3.0 to 0.4.0 (#652) (4 weeks ago) <dependabot[bot]>
* 37b0192 - perf(interpreter): improve i256 instructions (#630) (4 weeks ago) <DaniPopes>
* 214e65d - chore(interpreter): improve gas calculations (#632) (5 weeks ago) <DaniPopes>
* 6b55b9c - feat(`interpreter`): add hash to bytecode (#628) (5 weeks ago) <evalir>
* 84a5e97 - chore(interpreter): use `let else` (#629) (5 weeks ago) <DaniPopes>
* e9d96cd - chore(interpreter): improve dummy host (#631) (5 weeks ago) <DaniPopes>
* 2054293 - chore: misc improvements (#633) (5 weeks ago) <DaniPopes>
* 68820da - feat(state): Block hash cache and overrides (#621) (5 weeks ago) <rakita>
* eb6a9f0 - Revert "feat: alloy migration (#535)" (#616) (6 weeks ago) <rakita>
* c1bad0d - chore: spell check (#615) (6 weeks ago) <Roman Krasiuk>
* f95b7a4 - feat: alloy migration (#535) (6 weeks ago) <DaniPopes>
* bc4d203 - feat: remove unnecessary var and if branch in gas calc (#592) (7 weeks ago) <bemevolent>
* ef57a46 - feat: State with account status (#499) (7 weeks ago) <rakita>
* 157ef36 - feat: introduce initcode size limit check taking config into account (#587) (7 weeks ago) <evalir>
* 12558c5 - fix: fix mcopy memory expansion. Add eth tests to ci (#586) (7 weeks ago) <rakita>
* 06b1f6b - feat: EIP-1153 Transient storage opcodes (#546) (8 weeks ago) <Mark Tyneway>
* c6c5e88 - make calc public  (#575) (8 weeks ago) <BrazilRaw>
* 0a739e4 - fix(interpreter): mcopy call order (#570) (8 weeks ago) <DaniPopes>
* 30bfa73 - fix(doc): Inline documentation of re-exports (#560) (9 weeks ago) <Yiannis Marangos>
* 36de35b - feat: Rename all SHA3 opcodes to KECCAK256 (#514) (3 months ago) <Tung Bui (Leo)>
* 10f81ba - optimize stack usage for recursive `call` and `create` programs (#522) (3 months ago) <Valentin Mihov>
* c153428 - feat(cancun): EIP-5656: MCOPY - Memory copying instruction (#528) (3 months ago) <Waylon Jepsen>
* 51072e6 - consume all gas on invalid opcode (#500) (3 months ago) <teddav>
* ccd0298 - feat: add Memory::into_data (#516) (3 months ago) <Matthias Seitz>
* 69f417f - feat: simplify BYTE opcode (#512) (4 months ago) <teddav>
* c54f079 - fix: replace SHA3 with KECCAK256 opcode name (#511) (4 months ago) <Matthias Seitz>
* f8ff6b3 - feat: separate initial checks (#486) (5 months ago) <rakita>
* 6057cc2 - chore: refactor interpreter run and remove static flag (#481) (5 months ago) <rakita>


# v1.1.2
date: 03.05.2023

* 08091e1 - fix: compile errors for features (#467) (13 days ago) <rakita>

# v1.1.1
date: 14.04.2023

Added back utility function:
* 7d9b38a - [Interpreter]: Add back `spec_gas_opcode` (#446) (9 days ago) <Enrique Ortiz>

# v1.1.0
date: 04.04.2023

Biggest changes are Shanghai support 08ce847 and removal of gas blocks f91d5f9.

Changelog:
* c2ee8ff - add feature for ignoring base fee check (#436) (6 days ago) <Dan Cline>
* 0eff6a7 - Fix panic! message (#431) (2 weeks ago) <David Kulman>
* d0038e3 - chore(deps): bump arbitrary from 1.2.3 to 1.3.0 (#428) (2 weeks ago) <dependabot[bot]>
* dd0e227 - feat: Add all internals results to Halt (#413) (4 weeks ago) <rakita>
* d8dc652 - fix(interpreter): halt on CreateInitcodeSizeLimit (#412) (4 weeks ago) <Roman Krasiuk>
* a193d79 - chore: enabled primtive default feature in precompile (#409) (4 weeks ago) <Matthias Seitz>
* 1720729 - chore: add display impl for Opcode (#406) (4 weeks ago) <Matthias Seitz>
* 33bf8a8 - feat: use singular bytes for the jumpmap (#402) (4 weeks ago) <Bjerg>
* 394e8e9 - feat: extend SuccessOrHalt (#405) (4 weeks ago) <Matthias Seitz>
* f91d5f9 - refactor: remove gas blocks (#391) (5 weeks ago) <Bjerg>
* a8ae3f4 - fix: using pop_top instead of pop in eval_exp (#379) (7 weeks ago) <flyq>
* 08ce847 - feat(Shanghai): All EIPs: push0, warm coinbase, limit/measure initcode (#376) (7 weeks ago) <rakita>
* 6710511 - add no_std to primitives (#366) (7 weeks ago) <rakita>
* 1fca102 - chore(deps): bump proptest from 1.0.0 to 1.1.0 (#358) (8 weeks ago) <dependabot[bot]>
* 9b663bb - feat: Different OutOfGas Error types (#354) (9 weeks ago) <Chirag Baghasingh>

# v1.0.0
date: 29.01.2023

Interpreter was extracted from main revm crate at the revm v3.0.0 version.