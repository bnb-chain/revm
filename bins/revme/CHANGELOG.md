# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.0](https://github.com/bnb-chain/revm/compare/revme-v0.8.0...revme-v0.9.0) - 2024-08-07

### Added
- *(eof)* cli eof-validation ([#1622](https://github.com/bnb-chain/revm/pull/1622))
- *(EOF)* Bytecode::new_raw supports EOF, new_raw_checked added ([#1607](https://github.com/bnb-chain/revm/pull/1607))
- *(EOF)* Put EOF bytecode behind an Arc ([#1517](https://github.com/bnb-chain/revm/pull/1517))
- *(revme)* add prague spec ([#1506](https://github.com/bnb-chain/revm/pull/1506))
- *(precompile)* Prague - EIP-2537 - BLS12-381 curve operations ([#1389](https://github.com/bnb-chain/revm/pull/1389))
- add trace option in `revme evm` ([#1376](https://github.com/bnb-chain/revm/pull/1376))
- *(revme)* add --keep-going to statetest command ([#1277](https://github.com/bnb-chain/revm/pull/1277))
- EOF (Ethereum Object Format) ([#1143](https://github.com/bnb-chain/revm/pull/1143))
- [**breaking**] TracerEip3155 optionally traces memory ([#1234](https://github.com/bnb-chain/revm/pull/1234))
- use `impl` instead of `dyn` in `GetInspector` ([#1157](https://github.com/bnb-chain/revm/pull/1157))
- add evm script ([#1039](https://github.com/bnb-chain/revm/pull/1039))
- split off serde_json dependency to its own feature ([#1104](https://github.com/bnb-chain/revm/pull/1104))
- tweeks for v4.0 revm release ([#1048](https://github.com/bnb-chain/revm/pull/1048))
- *(revme)* make it runnable by goevmlab ([#990](https://github.com/bnb-chain/revm/pull/990))
- EvmBuilder and External Contexts ([#888](https://github.com/bnb-chain/revm/pull/888))
- Loop call stack ([#851](https://github.com/bnb-chain/revm/pull/851))
- *(revme)* format kzg setup ([#818](https://github.com/bnb-chain/revm/pull/818))
- *(interpreter)* add more helper methods to memory ([#794](https://github.com/bnb-chain/revm/pull/794))
- derive more traits ([#745](https://github.com/bnb-chain/revm/pull/745))
- Alloy primitives ([#724](https://github.com/bnb-chain/revm/pull/724))
- implement EIP-4844 ([#668](https://github.com/bnb-chain/revm/pull/668))
- *(StateBuilder)* switch builder option from without_bundle to with_bundle ([#688](https://github.com/bnb-chain/revm/pull/688))
- alloy migration ([#535](https://github.com/bnb-chain/revm/pull/535))
- State with account status ([#499](https://github.com/bnb-chain/revm/pull/499))
- *(cancun)* EIP-5656: MCOPY - Memory copying instruction ([#528](https://github.com/bnb-chain/revm/pull/528))
- json opcode traces EIP-3155 ([#356](https://github.com/bnb-chain/revm/pull/356))
- *(Shanghai)* All EIPs: push0, warm coinbase, limit/measure initcode ([#376](https://github.com/bnb-chain/revm/pull/376))
- revm-interpreter created ([#320](https://github.com/bnb-chain/revm/pull/320))
- Export CustomPrinter insector from revm ([#300](https://github.com/bnb-chain/revm/pull/300))
- substitute web3db to ethersdb ([#293](https://github.com/bnb-chain/revm/pull/293))
- *(interpreter)* Unify instruction fn signature ([#283](https://github.com/bnb-chain/revm/pull/283))
- *(revm)* Add prevrandao field to EnvBlock ([#271](https://github.com/bnb-chain/revm/pull/271))
- Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` ([#239](https://github.com/bnb-chain/revm/pull/239))
- *(revm, revme)* gas inspector ([#222](https://github.com/bnb-chain/revm/pull/222))
- Introduce ByteCode format, Update Readme ([#156](https://github.com/bnb-chain/revm/pull/156))
- mutable call inputs

### Fixed
- *(eip7702)* Add tests and fix some bugs ([#1605](https://github.com/bnb-chain/revm/pull/1605))
- *(eof)* fixture 2 tests ([#1550](https://github.com/bnb-chain/revm/pull/1550))
- *(revme)* Print one json outcome in statetest ([#1347](https://github.com/bnb-chain/revm/pull/1347))
- Drops check for .json when testing a single file ([#1301](https://github.com/bnb-chain/revm/pull/1301))
- *(revme)* revme error output and remove double summary ([#1169](https://github.com/bnb-chain/revm/pull/1169))
- *(eip4844)* Pass eth tests, additional conditions added. ([#735](https://github.com/bnb-chain/revm/pull/735))
- *(test)* Check expect exception and revm error ([#734](https://github.com/bnb-chain/revm/pull/734))
- k256 compile error ([#451](https://github.com/bnb-chain/revm/pull/451))
- make DatabaseRef::basic consistent with Database ([#201](https://github.com/bnb-chain/revm/pull/201))
- impose a memory limit ([#86](https://github.com/bnb-chain/revm/pull/86))
- various inspector fixes ([#69](https://github.com/bnb-chain/revm/pull/69))

### Other
- bump versions bcs of primitives ([#1631](https://github.com/bnb-chain/revm/pull/1631))
- release ([#1620](https://github.com/bnb-chain/revm/pull/1620))
- *(GeneralState)* skip fewer specs ([#1603](https://github.com/bnb-chain/revm/pull/1603))
- Merge branch 'refs/heads/develop' into merge-v11.0.0
- release ([#1579](https://github.com/bnb-chain/revm/pull/1579))
- replace AccessList with alloy version ([#1552](https://github.com/bnb-chain/revm/pull/1552))
- release ([#1548](https://github.com/bnb-chain/revm/pull/1548))
- replace TransactTo with TxKind ([#1542](https://github.com/bnb-chain/revm/pull/1542))
- skip tests with storage check and return status ([#1452](https://github.com/bnb-chain/revm/pull/1452))
- release ([#1261](https://github.com/bnb-chain/revm/pull/1261))
- *(revme)* increment statetest bar *after* running the test ([#1377](https://github.com/bnb-chain/revm/pull/1377))
- *(interpreter)* branch less in as_usize_or_fail ([#1374](https://github.com/bnb-chain/revm/pull/1374))
- release ([#1231](https://github.com/bnb-chain/revm/pull/1231))
- use uint macro & fix various small things ([#1253](https://github.com/bnb-chain/revm/pull/1253))
- release ([#1175](https://github.com/bnb-chain/revm/pull/1175))
- tag v32 revm v7.1.0 ([#1176](https://github.com/bnb-chain/revm/pull/1176))
- release ([#1125](https://github.com/bnb-chain/revm/pull/1125))
- *(deps)* bump walkdir from 2.4.0 to 2.5.0 ([#1149](https://github.com/bnb-chain/revm/pull/1149))
- release tag v30 revm v6.1.0 ([#1100](https://github.com/bnb-chain/revm/pull/1100))
- release ([#1082](https://github.com/bnb-chain/revm/pull/1082))
- license date and revm docs ([#1080](https://github.com/bnb-chain/revm/pull/1080))
- release ([#1067](https://github.com/bnb-chain/revm/pull/1067))
- *(revme)* statetests new format and return error ([#1066](https://github.com/bnb-chain/revm/pull/1066))
- tag v27, revm v4.0.0 release ([#1061](https://github.com/bnb-chain/revm/pull/1061))
- *(EvmBuilder)* rename builder functions to HandlerCfg ([#1050](https://github.com/bnb-chain/revm/pull/1050))
- *(Interpreter)* Split calls to separate functions ([#1005](https://github.com/bnb-chain/revm/pull/1005))
- *(revme)* EmptyDb Blockhash string, json-outcome flag, set prevrandao in statetest ([#994](https://github.com/bnb-chain/revm/pull/994))
- *(revme)* add recovery of address from secret key ([#992](https://github.com/bnb-chain/revm/pull/992))
- *(log)* use alloy_primitives::Log ([#975](https://github.com/bnb-chain/revm/pull/975))
- *(docs)* revme readme update ([#898](https://github.com/bnb-chain/revm/pull/898))
- simplify use statements ([#864](https://github.com/bnb-chain/revm/pull/864))
- decode KZG points directly into the buffers ([#840](https://github.com/bnb-chain/revm/pull/840))
- bump v26 revm v3.5.0 ([#765](https://github.com/bnb-chain/revm/pull/765))
- tag v25, revm v3.4.0 ([#755](https://github.com/bnb-chain/revm/pull/755))
- BLOBBASEFEE opcode ([#721](https://github.com/bnb-chain/revm/pull/721))
- Never inline the prepare functions ([#712](https://github.com/bnb-chain/revm/pull/712))
- *(deps)* bump bytes from 1.4.0 to 1.5.0 ([#707](https://github.com/bnb-chain/revm/pull/707))
- make `impl Default for StateBuilder` generic ([#690](https://github.com/bnb-chain/revm/pull/690))
- *(deps)* bump walkdir from 2.3.3 to 2.4.0 ([#692](https://github.com/bnb-chain/revm/pull/692))
- *(cfg)* convert chain_id from u256 to u64 ([#693](https://github.com/bnb-chain/revm/pull/693))
- Revert "feat: alloy migration ([#535](https://github.com/bnb-chain/revm/pull/535))" ([#616](https://github.com/bnb-chain/revm/pull/616))
- spell check ([#615](https://github.com/bnb-chain/revm/pull/615))
- avoid unnecessary allocations ([#581](https://github.com/bnb-chain/revm/pull/581))
- clippy and fmt ([#568](https://github.com/bnb-chain/revm/pull/568))
- optimize stack usage for recursive `call` and `create` programs ([#522](https://github.com/bnb-chain/revm/pull/522))
- *(deps)* bump hashbrown from 0.13.2 to 0.14.0 ([#519](https://github.com/bnb-chain/revm/pull/519))
- Bump v24, revm v3.3.0 ([#476](https://github.com/bnb-chain/revm/pull/476))
- *(deps)* bump ruint from 1.7.0 to 1.8.0 ([#465](https://github.com/bnb-chain/revm/pull/465))
- Release v23, revm v3.2.0 ([#464](https://github.com/bnb-chain/revm/pull/464))
- Release v22, revm v3.1.1 ([#460](https://github.com/bnb-chain/revm/pull/460))
- v21, revm v3.1.0 ([#444](https://github.com/bnb-chain/revm/pull/444))
- bump all
- remove gas blocks ([#391](https://github.com/bnb-chain/revm/pull/391))
- *(deps)* bump bytes from 1.3.0 to 1.4.0 ([#355](https://github.com/bnb-chain/revm/pull/355))
- Bump v20, changelog ([#350](https://github.com/bnb-chain/revm/pull/350))
- Cleanup imports ([#348](https://github.com/bnb-chain/revm/pull/348))
- includes to libs ([#338](https://github.com/bnb-chain/revm/pull/338))
- Creating revm-primitives, revm better errors and db components  ([#334](https://github.com/bnb-chain/revm/pull/334))
- Correct typo ([#282](https://github.com/bnb-chain/revm/pull/282))
- Integer overflow while calculating the remaining gas in GasInspector ([#287](https://github.com/bnb-chain/revm/pull/287))
- native bits ([#278](https://github.com/bnb-chain/revm/pull/278))
- *(release)* Bump revm and precompiles versions
- Bump primitive_types. Add statetest spec
- Bump revm to v2.3.0
- typos ([#263](https://github.com/bnb-chain/revm/pull/263))
- *(eth/test)* Added OEF spec for tests. Skip HighGasPrice ([#261](https://github.com/bnb-chain/revm/pull/261))
- Bump revm v2.1.0 ([#224](https://github.com/bnb-chain/revm/pull/224))
- revm bump v2.0.0, precompile bump v1.1.1 ([#212](https://github.com/bnb-chain/revm/pull/212))
- current_opcode fn and rename program_counter to instruction_pointer ([#211](https://github.com/bnb-chain/revm/pull/211))
- Cfg choose create analysis, option on bytecode size limit ([#210](https://github.com/bnb-chain/revm/pull/210))
- revme some cleanup ([#202](https://github.com/bnb-chain/revm/pull/202))
- Add support for old forks. ([#191](https://github.com/bnb-chain/revm/pull/191))
- add lib target, make utils public ([#185](https://github.com/bnb-chain/revm/pull/185))
- Handle HighNonce tests ([#176](https://github.com/bnb-chain/revm/pull/176))
- JournaledState ([#175](https://github.com/bnb-chain/revm/pull/175))
- Return `ExecutionResult`, which includes `gas_refunded` ([#169](https://github.com/bnb-chain/revm/pull/169))
- Make CacheDB fields pub ([#145](https://github.com/bnb-chain/revm/pull/145))
- Introduce account Touched/Cleared/None state in CacheDB ([#140](https://github.com/bnb-chain/revm/pull/140))
- update statetest model to pass merge tests ([#133](https://github.com/bnb-chain/revm/pull/133))
- don't delete account and storage entries on commit ([#126](https://github.com/bnb-chain/revm/pull/126))
- *(clippy)* make clippy happy ([#120](https://github.com/bnb-chain/revm/pull/120))
- typo fixes
- v6 changelog, bump versions
- some cleanup, checking on failed example tests
- Rework analysis ([#89](https://github.com/bnb-chain/revm/pull/89))
- refactor to exact option combinators ([#96](https://github.com/bnb-chain/revm/pull/96))
- Enable statetest for Berlin/Istanbul ([#78](https://github.com/bnb-chain/revm/pull/78))
- Big Refactor. Machine to Interpreter. refactor instructions. call/create struct ([#52](https://github.com/bnb-chain/revm/pull/52))
- [revm] pop_top and unsafe comments ([#51](https://github.com/bnb-chain/revm/pull/51))
- Inspector fixup
- Bump precompiles to v0.4.0 bump revm v1.2.0
- [revme] return error on failes statetest
- clippy
- [recompl] Bump precompile deps, cargo sort on workspace
- cargo fmt
- [revm_precompiles] added flag for k256 lib
- [revm] Bump to v1.1.0
- Omit edgecase high nonce test. tracer gas fix
- Bug fix for unknown OpCode
- internal cleanups
- [revm] output log. Stetetest test log output. fmt
- Bump versions, Changelogs, fmt, revm readme, clippy.
- GasBlock for all Spec
- [revm] Run test multiple times. fmt, BenchmarkDB
- [revm][perf] GasBlock analazis and optimizations.
- wip
- [revm] Optimize PC, some perf
- [revme][debug] added help ctrl
- [revme][debugger] stack pop/push
- [revme] full env as cli
- [revme][debug] some print cli
- readme. debuger update
- [revm] Rename Handler to Host
- [revm] Simplified host inspector
- [revme] debugger cli history
- [revme][debugger] wip terminal
- [revm][revme] statetest merged
- [revme] initial commit. Cmd skeleton added.statetests moved

## [0.8.0](https://github.com/bluealloy/revm/compare/revme-v0.7.0...revme-v0.8.0) - 2024-07-16

### Added
- *(eof)* cli eof-validation ([#1622](https://github.com/bluealloy/revm/pull/1622))
- *(EOF)* Bytecode::new_raw supports EOF, new_raw_checked added ([#1607](https://github.com/bluealloy/revm/pull/1607))

### Fixed
- *(eip7702)* Add tests and fix some bugs ([#1605](https://github.com/bluealloy/revm/pull/1605))

### Other
- *(GeneralState)* skip fewer specs ([#1603](https://github.com/bluealloy/revm/pull/1603))

## [0.7.0](https://github.com/bluealloy/revm/compare/revme-v0.6.0...revme-v0.7.0) - 2024-07-08

### Other
- replace AccessList with alloy version ([#1552](https://github.com/bluealloy/revm/pull/1552))

## [0.6.0](https://github.com/bluealloy/revm/compare/revme-v0.5.0...revme-v0.6.0) - 2024-06-20

### Added
- *(EOF)* Put EOF bytecode behind an Arc ([#1517](https://github.com/bluealloy/revm/pull/1517))
- *(revme)* add prague spec ([#1506](https://github.com/bluealloy/revm/pull/1506))

### Fixed
- *(eof)* fixture 2 tests ([#1550](https://github.com/bluealloy/revm/pull/1550))

### Other
- replace TransactTo with TxKind ([#1542](https://github.com/bluealloy/revm/pull/1542))
- skip tests with storage check and return status ([#1452](https://github.com/bluealloy/revm/pull/1452))

## [0.5.0](https://github.com/bluealloy/revm/compare/revme-v0.4.0...revme-v0.5.0) - 2024-05-12

### Added
- *(precompile)* Prague - EIP-2537 - BLS12-381 curve operations ([#1389](https://github.com/bluealloy/revm/pull/1389))
- add trace option in `revme evm` ([#1376](https://github.com/bluealloy/revm/pull/1376))
- *(revme)* add --keep-going to statetest command ([#1277](https://github.com/bluealloy/revm/pull/1277))
- EOF (Ethereum Object Format) ([#1143](https://github.com/bluealloy/revm/pull/1143))

### Fixed
- *(revme)* Print one json outcome in statetest ([#1347](https://github.com/bluealloy/revm/pull/1347))
- Drops check for .json when testing a single file ([#1301](https://github.com/bluealloy/revm/pull/1301))

### Other
- *(revme)* increment statetest bar *after* running the test ([#1377](https://github.com/bluealloy/revm/pull/1377))
- *(interpreter)* branch less in as_usize_or_fail ([#1374](https://github.com/bluealloy/revm/pull/1374))

## [0.4.0](https://github.com/bluealloy/revm/compare/revme-v0.3.1...revme-v0.4.0) - 2024-04-02

### Added
- [**breaking**] TracerEip3155 optionally traces memory ([#1234](https://github.com/bluealloy/revm/pull/1234))

### Other
- use uint macro & fix various small things ([#1253](https://github.com/bluealloy/revm/pull/1253))

## [0.3.1](https://github.com/bluealloy/revm/compare/revme-v0.3.0...revme-v0.3.1) - 2024-03-19

### Other
- tag v32 revm v7.1.0 ([#1176](https://github.com/bluealloy/revm/pull/1176))

## [0.3.0](https://github.com/bluealloy/revm/compare/revme-v0.2.2...revme-v0.3.0) - 2024-03-08

### Added
- use `impl` instead of `dyn` in `GetInspector` ([#1157](https://github.com/bluealloy/revm/pull/1157))
- add evm script ([#1039](https://github.com/bluealloy/revm/pull/1039))

### Fixed
- *(revme)* revme error output and remove double summary ([#1169](https://github.com/bluealloy/revm/pull/1169))

### Other
- *(deps)* bump walkdir from 2.4.0 to 2.5.0 ([#1149](https://github.com/bluealloy/revm/pull/1149))

## [0.2.2](https://github.com/bluealloy/revm/compare/revme-v0.2.1...revme-v0.2.2) - 2024-02-22

### Added
- split off serde_json dependency to its own feature ([#1104](https://github.com/bluealloy/revm/pull/1104))

## [0.2.1](https://github.com/bluealloy/revm/compare/revme-v0.2.0...revme-v0.2.1) - 2024-02-07

### Added
- tweeks for v4.0 revm release ([#1048](https://github.com/bluealloy/revm/pull/1048))
- *(revme)* make it runnable by goevmlab ([#990](https://github.com/bluealloy/revm/pull/990))
- EvmBuilder and External Contexts ([#888](https://github.com/bluealloy/revm/pull/888))
- Loop call stack ([#851](https://github.com/bluealloy/revm/pull/851))
- *(revme)* format kzg setup ([#818](https://github.com/bluealloy/revm/pull/818))
- *(interpreter)* add more helper methods to memory ([#794](https://github.com/bluealloy/revm/pull/794))
- derive more traits ([#745](https://github.com/bluealloy/revm/pull/745))
- Alloy primitives ([#724](https://github.com/bluealloy/revm/pull/724))
- implement EIP-4844 ([#668](https://github.com/bluealloy/revm/pull/668))
- *(StateBuilder)* switch builder option from without_bundle to with_bundle ([#688](https://github.com/bluealloy/revm/pull/688))
- alloy migration ([#535](https://github.com/bluealloy/revm/pull/535))
- State with account status ([#499](https://github.com/bluealloy/revm/pull/499))
- *(cancun)* EIP-5656: MCOPY - Memory copying instruction ([#528](https://github.com/bluealloy/revm/pull/528))
- json opcode traces EIP-3155 ([#356](https://github.com/bluealloy/revm/pull/356))
- *(Shanghai)* All EIPs: push0, warm coinbase, limit/measure initcode ([#376](https://github.com/bluealloy/revm/pull/376))
- revm-interpreter created ([#320](https://github.com/bluealloy/revm/pull/320))
- Export CustomPrinter insector from revm ([#300](https://github.com/bluealloy/revm/pull/300))
- substitute web3db to ethersdb ([#293](https://github.com/bluealloy/revm/pull/293))
- *(interpreter)* Unify instruction fn signature ([#283](https://github.com/bluealloy/revm/pull/283))
- *(revm)* Add prevrandao field to EnvBlock ([#271](https://github.com/bluealloy/revm/pull/271))
- Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` ([#239](https://github.com/bluealloy/revm/pull/239))
- *(revm, revme)* gas inspector ([#222](https://github.com/bluealloy/revm/pull/222))

### Fixed
- *(eip4844)* Pass eth tests, additional conditions added. ([#735](https://github.com/bluealloy/revm/pull/735))
- *(test)* Check expect exception and revm error ([#734](https://github.com/bluealloy/revm/pull/734))
- k256 compile error ([#451](https://github.com/bluealloy/revm/pull/451))

### Other
- *(EvmBuilder)* rename builder functions to HandlerCfg ([#1050](https://github.com/bluealloy/revm/pull/1050))
- *(Interpreter)* Split calls to separate functions ([#1005](https://github.com/bluealloy/revm/pull/1005))
- *(revme)* EmptyDb Blockhash string, json-outcome flag, set prevrandao in statetest ([#994](https://github.com/bluealloy/revm/pull/994))
- *(revme)* add recovery of address from secret key ([#992](https://github.com/bluealloy/revm/pull/992))
- *(log)* use alloy_primitives::Log ([#975](https://github.com/bluealloy/revm/pull/975))
- *(docs)* revme readme update ([#898](https://github.com/bluealloy/revm/pull/898))
- simplify use statements ([#864](https://github.com/bluealloy/revm/pull/864))
- decode KZG points directly into the buffers ([#840](https://github.com/bluealloy/revm/pull/840))
- bump v26 revm v3.5.0 ([#765](https://github.com/bluealloy/revm/pull/765))
- tag v25, revm v3.4.0 ([#755](https://github.com/bluealloy/revm/pull/755))
- BLOBBASEFEE opcode ([#721](https://github.com/bluealloy/revm/pull/721))
- Never inline the prepare functions ([#712](https://github.com/bluealloy/revm/pull/712))
- *(deps)* bump bytes from 1.4.0 to 1.5.0 ([#707](https://github.com/bluealloy/revm/pull/707))
- make `impl Default for StateBuilder` generic ([#690](https://github.com/bluealloy/revm/pull/690))
- *(deps)* bump walkdir from 2.3.3 to 2.4.0 ([#692](https://github.com/bluealloy/revm/pull/692))
- *(cfg)* convert chain_id from u256 to u64 ([#693](https://github.com/bluealloy/revm/pull/693))
- Revert "feat: alloy migration ([#535](https://github.com/bluealloy/revm/pull/535))" ([#616](https://github.com/bluealloy/revm/pull/616))
- spell check ([#615](https://github.com/bluealloy/revm/pull/615))
- avoid unnecessary allocations ([#581](https://github.com/bluealloy/revm/pull/581))
- clippy and fmt ([#568](https://github.com/bluealloy/revm/pull/568))
- optimize stack usage for recursive `call` and `create` programs ([#522](https://github.com/bluealloy/revm/pull/522))
- *(deps)* bump hashbrown from 0.13.2 to 0.14.0 ([#519](https://github.com/bluealloy/revm/pull/519))
- Bump v24, revm v3.3.0 ([#476](https://github.com/bluealloy/revm/pull/476))
- *(deps)* bump ruint from 1.7.0 to 1.8.0 ([#465](https://github.com/bluealloy/revm/pull/465))
- Release v23, revm v3.2.0 ([#464](https://github.com/bluealloy/revm/pull/464))
- Release v22, revm v3.1.1 ([#460](https://github.com/bluealloy/revm/pull/460))
- v21, revm v3.1.0 ([#444](https://github.com/bluealloy/revm/pull/444))
- bump all
- remove gas blocks ([#391](https://github.com/bluealloy/revm/pull/391))
- *(deps)* bump bytes from 1.3.0 to 1.4.0 ([#355](https://github.com/bluealloy/revm/pull/355))
- Bump v20, changelog ([#350](https://github.com/bluealloy/revm/pull/350))
- Cleanup imports ([#348](https://github.com/bluealloy/revm/pull/348))
- includes to libs ([#338](https://github.com/bluealloy/revm/pull/338))
- Creating revm-primitives, revm better errors and db components  ([#334](https://github.com/bluealloy/revm/pull/334))
- Correct typo ([#282](https://github.com/bluealloy/revm/pull/282))
- Integer overflow while calculating the remaining gas in GasInspector ([#287](https://github.com/bluealloy/revm/pull/287))
- native bits ([#278](https://github.com/bluealloy/revm/pull/278))
- *(release)* Bump revm and precompiles versions
- Bump primitive_types. Add statetest spec
- Bump revm to v2.3.0
- typos ([#263](https://github.com/bluealloy/revm/pull/263))
- *(eth/test)* Added OEF spec for tests. Skip HighGasPrice ([#261](https://github.com/bluealloy/revm/pull/261))
- Bump revm v2.1.0 ([#224](https://github.com/bluealloy/revm/pull/224))
# v0.1.0
date: 18.12.2021

Initial release. statetest are done, other things I have just started working on.