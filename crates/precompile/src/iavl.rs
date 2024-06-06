use crate::{Bytes, Error, Precompile, PrecompileError, PrecompileResult, PrecompileWithAddress};
use parity_bytes::BytesRef;
use tendermint::lite::iavl_proof;

/// Iavl proof validation precompile for BSC.
pub(crate) const IAVL_PROOF_VALIDATION: PrecompileWithAddress = PrecompileWithAddress(
    crate::u64_to_address(101),
    Precompile::Standard(crate::iavl::iavl_proof_validation_run),
);

/// Iavl proof validation precompile for BSC after Nano hardfork.
pub(crate) const IAVL_PROOF_VALIDATION_NANO: PrecompileWithAddress = PrecompileWithAddress(
    crate::u64_to_address(101),
    Precompile::Standard(crate::iavl::iavl_proof_validation_run_nano),
);

/// Iavl proof validation precompile for BSC after Moran hardfork.
pub(crate) const IAVL_PROOF_VALIDATION_MORAN: PrecompileWithAddress = PrecompileWithAddress(
    crate::u64_to_address(101),
    Precompile::Standard(crate::iavl::iavl_proof_validation_run_moran),
);

/// Iavl proof validation precompile for BSC after Planck hardfork.
pub(crate) const IAVL_PROOF_VALIDATION_PLANCK: PrecompileWithAddress = PrecompileWithAddress(
    crate::u64_to_address(101),
    Precompile::Standard(crate::iavl::iavl_proof_validation_run_planck),
);

/// Iavl proof validation precompile for BSC after Plato hardfork.
pub(crate) const IAVL_PROOF_VALIDATION_PLATO: PrecompileWithAddress = PrecompileWithAddress(
    crate::u64_to_address(101),
    Precompile::Standard(crate::iavl::iavl_proof_validation_run_plato),
);

/// Run Iavl proof validation.
fn iavl_proof_validation_run(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    return iavl_proof_validation_run_inner(input, gas_limit, false, false, false);
}

/// Run Iavl proof validation with Nano hardfork.
fn iavl_proof_validation_run_nano(_input: &Bytes, _gas_limit: u64) -> PrecompileResult {
    return Err(PrecompileError::other("suspended"));
}

/// Run Iavl proof validation with Moran hardfork.
fn iavl_proof_validation_run_moran(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    return iavl_proof_validation_run_inner(input, gas_limit, true, false, false);
}

/// Run Iavl proof validation with Planck hardfork.
fn iavl_proof_validation_run_planck(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    return iavl_proof_validation_run_inner(input, gas_limit, false, true, false);
}

/// Run Iavl proof validation with Plato hardfork.
fn iavl_proof_validation_run_plato(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    return iavl_proof_validation_run_inner(input, gas_limit, false, false, true);
}

/// Run Iavl proof validation with given hardfork toggles.
fn iavl_proof_validation_run_inner(
    input: &Bytes,
    gas_limit: u64,
    is_moran: bool,
    is_planck: bool,
    is_plato: bool,
) -> PrecompileResult {
    const IAVL_PROOF_VALIDATION_BASE: u64 = 3_000;

    if IAVL_PROOF_VALIDATION_BASE > gas_limit {
        return Err(Error::OutOfGas);
    }

    let mut output = [0u8; 32];
    let mut bytes = BytesRef::Fixed(&mut output);
    let res = iavl_proof::execute(input.as_ref(), &mut bytes, is_moran, is_planck, is_plato);
    match res {
        Ok(()) => Ok((
            IAVL_PROOF_VALIDATION_BASE,
            Bytes::copy_from_slice(&output[..]),
        )),
        Err(str) => Err(PrecompileError::Other(String::from(str))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use revm_primitives::hex;

    #[test]
    fn test_iavl_proof_validation_run() {
        let input = hex::decode("00000000000000000000000000000000000000000000000000000000000007306163630000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001c6163636f756e743a8a4e2eb018bdf98a8f53ec755740ffc728637a1d000000000000000000000000000000000000000000000000000000000000007b4bdc4c270a750a148a4e2eb018bdf98a8f53ec755740ffc728637a1d12110a0941544348412d3733301080f69bf321120b0a03424e4210e8baeb8d44120f0a075050432d303041108094ebdc031a26eb5ae98721031c199c92e5b0080967da99be27cf2da53317441b4a663e6d9c6caf02be1fdbdc20d7962b28152c69c314b4de5c8035253c8bc0771d9ca17b1b23a57c0c6d068b57579791cae20add070a066961766c3a76121c6163636f756e743a8a4e2eb018bdf98a8f53ec755740ffc728637a1d1ab407b2070aaf070a2d081810cdfd2b188096a82222209f223f804e2d94ac51c4321b0687397012e6d95eb9783b03bc790da631004c7c0a2d081710adb31a18f395a8222a20d2a38865de82383ccce0140513b65cec1bf2ae6cd7dfeb22eb6faadb4e26b26f0a2d081510b2990b18f395a82222208a02bbd5a695dfc772627ac8744aa9cf30ae26575bdce8c96a9a0d0999175b430a2d081410e6ff0418f395a8222a20d39619c779be909e67f23499fb74eb2c19afd7f21523401d4ccf7e917db5cd600a2d081210e3fe0118f395a8222a20a10cc73843f889d9e03a463eb135e928bb980e19734344cba0fbf4e8a4c5258b0a2c081010dd6518f395a8222a2007fd15843a2fd3f58d021b0e072a6c70742d7a3d993a922445e3491e1c14ee8e0a2c080f10cc2a18eda6a7222a20088942d7b30abd021d8e9505cc41313fad87c8c10a799f3b51018b7b2cfe4ad90a2c080d10b70d18eda6a7222a2091a37bc44d0c61e3752ddc59eb390355ab65e8a9fb453be4f0acec537f1ca14f0a2c080c10890818eda6a72222201cfc317855a06667c45812fe36efe33af05671dfe0d9b56b02662011af2e79e30a2c080b10ac0318c4b0ee212220aeb454a4b3243b6269a2fd8841dca9a951c53b30f1e27da91063dae7224402c70a2c080910e40118c4b0ee212a20441340a4de6498f861b97b3f3ad9603af055e5af51a0d96fff2ae28e3c5c6c9a0a2c0808108d0118c4b0ee212220ae32ea4b9ab7b53571da320e2815fd8b2c278124961cca4a1849a799842424450a2b0807104d18c4b0ee212220e2804c9b7f045ec0b4ab20920a937b82fda8b7a9ddd12b21637335b915cfda550a2b0806102418a5f4c7192a20ec85f22addedfc82c771af5b4c77544b7c1d7c5bbac33f2712dfba1045ebdbd00a2b0805101118a5f4c7192a2071ade34dcc447a0ba8adc603080633d15c06f3525830c86ebce35eca0a4921fc0a2b0804100c18a5f4c7192a205190bce93993e65b266a3417ed511df8897a812cb4b62569e5afcfbec10b69cd0a2b0803100618a5f4c7192220b76c6884f1d412ac10bfb3987fb7d26f0330b2a85539509ebc5c6bdec2f95d520a2b0802100418a5f4c71922206a285b4a4f9d1c687bbafa1f3649b6a6e32b1a85dd0402421210683e846cf0020a2b0801100218a5f4c7192220033b3f7c6dcb258b6e55545e7a4f51539447cd595eb8a2e373ba0015502da1051a450a1c6163636f756e743a8a4e2eb018bdf98a8f53ec755740ffc728637a1d12201a272295e94cf1d8090bdb019dde48e9dab026ad2c3e43aaa7e61cc954a9245d18a5f4c7190ab6040a0a6d756c746973746f726512036163631aa204a0040a9d040a300a0364657812290a27088496a822122038fc49f49648fec62acc434151a51eaa378c1b20a730a749548e36f1529422500a300a03676f7612290a27088496a8221220a78ce489bdf08b9ee869c184876e1623dc38b3e64a5cf1a0005f97976c64deac0a380a0b61746f6d69635f7377617012290a27088496a8221220544c2fa38f61e10a39ec00b3e724d5834761268bb455cdbf5843bcf1531f8fbc0a300a0376616c12290a27088496a82212201f71082c9f6f45fb456b2c00b41e50d2f662f2dfec3cb6965f19d214bf02f3980a0f0a046d61696e12070a05088496a8220a320a057374616b6512290a27088496a82212200dd467343c718f240e50b4feac42970fc8c1c69a018be955f9c27913ac1f8b3c0a300a0361636312290a27088496a8221220270c19ccc9c40c5176b3dfbd8af734c97a307e0dbd8df9e286dcd5d709f973ed0a330a06746f6b656e7312290a27088496a8221220c4f96eedf50c83964de9df013afec2e545012d92528b643a5166c828774187b60a320a05706169727312290a27088496a8221220351c55cfda84596ecd22ebc77013662aba97f81f19d9ef3d150213bb07c823060a360a0974696d655f6c6f636b12290a27088496a8221220e7adf5bd30ce022decf0e9341bf05c464ed70cdbc97423bd2bab8f3571e5179b0a330a06706172616d7312290a27088496a822122042a9dfc356ca435db131eb41fb1975c8482f2434537918665e530b0b4633b5f9").unwrap();
        let res = iavl_proof_validation_run(&Bytes::from(input), 3_000u64).unwrap();

        let gas = res.0;
        assert_eq!(gas, 3_000u64);

        let res = hex::encode(res.1);
        assert_eq!(
            res,
            "0000000000000000000000000000000000000000000000000000000000000001"
        )
    }

    #[test]
    fn test_iavl_proof_validation_run_moran() {
        let input = hex::decode("00000000000000000000000000000000000000000000000000000000000007306163630000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001c6163636f756e743a8a4e2eb018bdf98a8f53ec755740ffc728637a1d000000000000000000000000000000000000000000000000000000000000007b4bdc4c270a750a148a4e2eb018bdf98a8f53ec755740ffc728637a1d12110a0941544348412d3733301080f69bf321120b0a03424e4210e8baeb8d44120f0a075050432d303041108094ebdc031a26eb5ae98721031c199c92e5b0080967da99be27cf2da53317441b4a663e6d9c6caf02be1fdbdc20d7962b28152c69c314b4de5c8035253c8bc0771d9ca17b1b23a57c0c6d068b57579791cae20add070a066961766c3a76121c6163636f756e743a8a4e2eb018bdf98a8f53ec755740ffc728637a1d1ab407b2070aaf070a2d081810cdfd2b188096a82222209f223f804e2d94ac51c4321b0687397012e6d95eb9783b03bc790da631004c7c0a2d081710adb31a18f395a8222a20d2a38865de82383ccce0140513b65cec1bf2ae6cd7dfeb22eb6faadb4e26b26f0a2d081510b2990b18f395a82222208a02bbd5a695dfc772627ac8744aa9cf30ae26575bdce8c96a9a0d0999175b430a2d081410e6ff0418f395a8222a20d39619c779be909e67f23499fb74eb2c19afd7f21523401d4ccf7e917db5cd600a2d081210e3fe0118f395a8222a20a10cc73843f889d9e03a463eb135e928bb980e19734344cba0fbf4e8a4c5258b0a2c081010dd6518f395a8222a2007fd15843a2fd3f58d021b0e072a6c70742d7a3d993a922445e3491e1c14ee8e0a2c080f10cc2a18eda6a7222a20088942d7b30abd021d8e9505cc41313fad87c8c10a799f3b51018b7b2cfe4ad90a2c080d10b70d18eda6a7222a2091a37bc44d0c61e3752ddc59eb390355ab65e8a9fb453be4f0acec537f1ca14f0a2c080c10890818eda6a72222201cfc317855a06667c45812fe36efe33af05671dfe0d9b56b02662011af2e79e30a2c080b10ac0318c4b0ee212220aeb454a4b3243b6269a2fd8841dca9a951c53b30f1e27da91063dae7224402c70a2c080910e40118c4b0ee212a20441340a4de6498f861b97b3f3ad9603af055e5af51a0d96fff2ae28e3c5c6c9a0a2c0808108d0118c4b0ee212220ae32ea4b9ab7b53571da320e2815fd8b2c278124961cca4a1849a799842424450a2b0807104d18c4b0ee212220e2804c9b7f045ec0b4ab20920a937b82fda8b7a9ddd12b21637335b915cfda550a2b0806102418a5f4c7192a20ec85f22addedfc82c771af5b4c77544b7c1d7c5bbac33f2712dfba1045ebdbd00a2b0805101118a5f4c7192a2071ade34dcc447a0ba8adc603080633d15c06f3525830c86ebce35eca0a4921fc0a2b0804100c18a5f4c7192a205190bce93993e65b266a3417ed511df8897a812cb4b62569e5afcfbec10b69cd0a2b0803100618a5f4c7192220b76c6884f1d412ac10bfb3987fb7d26f0330b2a85539509ebc5c6bdec2f95d520a2b0802100418a5f4c71922206a285b4a4f9d1c687bbafa1f3649b6a6e32b1a85dd0402421210683e846cf0020a2b0801100218a5f4c7192220033b3f7c6dcb258b6e55545e7a4f51539447cd595eb8a2e373ba0015502da1051a450a1c6163636f756e743a8a4e2eb018bdf98a8f53ec755740ffc728637a1d12201a272295e94cf1d8090bdb019dde48e9dab026ad2c3e43aaa7e61cc954a9245d18a5f4c7190ab6040a0a6d756c746973746f726512036163631aa204a0040a9d040a300a0364657812290a27088496a822122038fc49f49648fec62acc434151a51eaa378c1b20a730a749548e36f1529422500a300a03676f7612290a27088496a8221220a78ce489bdf08b9ee869c184876e1623dc38b3e64a5cf1a0005f97976c64deac0a380a0b61746f6d69635f7377617012290a27088496a8221220544c2fa38f61e10a39ec00b3e724d5834761268bb455cdbf5843bcf1531f8fbc0a300a0376616c12290a27088496a82212201f71082c9f6f45fb456b2c00b41e50d2f662f2dfec3cb6965f19d214bf02f3980a0f0a046d61696e12070a05088496a8220a320a057374616b6512290a27088496a82212200dd467343c718f240e50b4feac42970fc8c1c69a018be955f9c27913ac1f8b3c0a300a0361636312290a27088496a8221220270c19ccc9c40c5176b3dfbd8af734c97a307e0dbd8df9e286dcd5d709f973ed0a330a06746f6b656e7312290a27088496a8221220c4f96eedf50c83964de9df013afec2e545012d92528b643a5166c828774187b60a320a05706169727312290a27088496a8221220351c55cfda84596ecd22ebc77013662aba97f81f19d9ef3d150213bb07c823060a360a0974696d655f6c6f636b12290a27088496a8221220e7adf5bd30ce022decf0e9341bf05c464ed70cdbc97423bd2bab8f3571e5179b0a330a06706172616d7312290a27088496a822122042a9dfc356ca435db131eb41fb1975c8482f2434537918665e530b0b4633b5f9").unwrap();
        let res = iavl_proof_validation_run_moran(&Bytes::from(input), 3_000u64).unwrap();

        let gas = res.0;
        assert_eq!(gas, 3_000u64);

        let res = hex::encode(res.1);
        assert_eq!(
            res,
            "0000000000000000000000000000000000000000000000000000000000000001"
        )
    }

    #[test]
    fn test_iavl_proof_validation_run_absence_proof_moran() {
        let input = hex::decode("00000000000000000000000000000000000000000000000000000000000007306163630000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001c6163636f756e743a8a4e2eb018bdf98a8f53ec755740ffc728637a1d000000000000000000000000000000000000000000000000000000000000007b4bdc4c270a750a148a4e2eb018bdf98a8f53ec755740ffc728637a1d12110a0941544348412d3733301080f69bf321120b0a03424e4210e8baeb8d44120f0a075050432d303041108094ebdc031a26eb5ae98721031c199c92e5b0080967da99be27cf2da53317441b4a663e6d9c6caf02be1fdbdc20d7962b28152c69c314b4de5c8035253c8bc0771d9ca17b1b23a57c0c6d068b57579791cae20add070a066961766c3a61121c6163636f756e743a8a4e2eb018bdf98a8f53ec755740ffc728637a1d1ab407b2070aaf070a2d081810cdfd2b188096a82222209f223f804e2d94ac51c4321b0687397012e6d95eb9783b03bc790da631004c7c0a2d081710adb31a18f395a8222a20d2a38865de82383ccce0140513b65cec1bf2ae6cd7dfeb22eb6faadb4e26b26f0a2d081510b2990b18f395a82222208a02bbd5a695dfc772627ac8744aa9cf30ae26575bdce8c96a9a0d0999175b430a2d081410e6ff0418f395a8222a20d39619c779be909e67f23499fb74eb2c19afd7f21523401d4ccf7e917db5cd600a2d081210e3fe0118f395a8222a20a10cc73843f889d9e03a463eb135e928bb980e19734344cba0fbf4e8a4c5258b0a2c081010dd6518f395a8222a2007fd15843a2fd3f58d021b0e072a6c70742d7a3d993a922445e3491e1c14ee8e0a2c080f10cc2a18eda6a7222a20088942d7b30abd021d8e9505cc41313fad87c8c10a799f3b51018b7b2cfe4ad90a2c080d10b70d18eda6a7222a2091a37bc44d0c61e3752ddc59eb390355ab65e8a9fb453be4f0acec537f1ca14f0a2c080c10890818eda6a72222201cfc317855a06667c45812fe36efe33af05671dfe0d9b56b02662011af2e79e30a2c080b10ac0318c4b0ee212220aeb454a4b3243b6269a2fd8841dca9a951c53b30f1e27da91063dae7224402c70a2c080910e40118c4b0ee212a20441340a4de6498f861b97b3f3ad9603af055e5af51a0d96fff2ae28e3c5c6c9a0a2c0808108d0118c4b0ee212220ae32ea4b9ab7b53571da320e2815fd8b2c278124961cca4a1849a799842424450a2b0807104d18c4b0ee212220e2804c9b7f045ec0b4ab20920a937b82fda8b7a9ddd12b21637335b915cfda550a2b0806102418a5f4c7192a20ec85f22addedfc82c771af5b4c77544b7c1d7c5bbac33f2712dfba1045ebdbd00a2b0805101118a5f4c7192a2071ade34dcc447a0ba8adc603080633d15c06f3525830c86ebce35eca0a4921fc0a2b0804100c18a5f4c7192a205190bce93993e65b266a3417ed511df8897a812cb4b62569e5afcfbec10b69cd0a2b0803100618a5f4c7192220b76c6884f1d412ac10bfb3987fb7d26f0330b2a85539509ebc5c6bdec2f95d520a2b0802100418a5f4c71922206a285b4a4f9d1c687bbafa1f3649b6a6e32b1a85dd0402421210683e846cf0020a2b0801100218a5f4c7192220033b3f7c6dcb258b6e55545e7a4f51539447cd595eb8a2e373ba0015502da1051a450a1c6163636f756e743a8a4e2eb018bdf98a8f53ec755740ffc728637a1d12201a272295e94cf1d8090bdb019dde48e9dab026ad2c3e43aaa7e61cc954a9245d18a5f4c7190ab6040a0a6d756c746973746f726512036163631aa204a0040a9d040a300a0364657812290a27088496a822122038fc49f49648fec62acc434151a51eaa378c1b20a730a749548e36f1529422500a300a03676f7612290a27088496a8221220a78ce489bdf08b9ee869c184876e1623dc38b3e64a5cf1a0005f97976c64deac0a380a0b61746f6d69635f7377617012290a27088496a8221220544c2fa38f61e10a39ec00b3e724d5834761268bb455cdbf5843bcf1531f8fbc0a300a0376616c12290a27088496a82212201f71082c9f6f45fb456b2c00b41e50d2f662f2dfec3cb6965f19d214bf02f3980a0f0a046d61696e12070a05088496a8220a320a057374616b6512290a27088496a82212200dd467343c718f240e50b4feac42970fc8c1c69a018be955f9c27913ac1f8b3c0a300a0361636312290a27088496a8221220270c19ccc9c40c5176b3dfbd8af734c97a307e0dbd8df9e286dcd5d709f973ed0a330a06746f6b656e7312290a27088496a8221220c4f96eedf50c83964de9df013afec2e545012d92528b643a5166c828774187b60a320a05706169727312290a27088496a8221220351c55cfda84596ecd22ebc77013662aba97f81f19d9ef3d150213bb07c823060a360a0974696d655f6c6f636b12290a27088496a8221220e7adf5bd30ce022decf0e9341bf05c464ed70cdbc97423bd2bab8f3571e5179b0a330a06706172616d7312290a27088496a822122042a9dfc356ca435db131eb41fb1975c8482f2434537918665e530b0b4633b5f9").unwrap();
        let res = iavl_proof_validation_run_moran(&Bytes::from(input), 3_000u64);

        assert_eq!(
            res.err(),
            Some(PrecompileError::Other(String::from("invalid merkle proof")))
        );
    }

    #[test]
    fn test_iavl_proof_validation_run_before_after_moran() {
        // Bytest1 is the inputs of exploit tx 0x05356fd06ce56a9ec5b4eaf9c075abd740cae4c21eab1676440ab5cd2fe5c57a
        let bytest1 = hex::decode("00000000000000000000000000000000000000000000000000000000000005086962630000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e00000100380200000000010dd9ac0000000000000000000000000000000000000000000000000000000000000093000000000000000000000000000000000000000000000000000000000000000000f870a0424e4200000000000000000000000000000000000000000000000000000000009400000000000000000000000000000000000000008ad3c21bcecceda100000094489a8756c18c0b8b24ec2a2b9ff3d4d447f79bec94489a8756c18c0b8b24ec2a2b9ff3d4d447f79bec846553f10072cda827a83531ca0fd7ac917a6b65649719aab0836722caafe0603147a523180a8d020a066961766c3a76120e00000100380200000000010dd9ac1af201f0010aed010a2b0802100318b091c73422200c10f902d266c238a4ca9e26fa9bc36483cd3ebee4e263012f5e7f40c22ee4d20a4d0801100218b091c7342220e4fd47bffd1c06e67edad92b2bf9ca63631978676288a2aa99f95c459436ef632a20121a1f9c4eca726c725796c5375fc4158986ced08e498dc8268ef94d8ed1891612001a370a0e0000010038020000000000000002122011056c6919f02d966991c10721684a8d1542e44003f9ffb47032c18995d4ac7f18b091c7341a340a0e00000100380200000000010dd9ac12202c3a561458f8527b002b5ec3cab2d308662798d6245d4588a4e6a80ebdfe30ac18010ad4050a0a6d756c746973746f726512036962631ac005be050abb050a110a066f7261636c6512070a0508b891c7340a0f0a046d61696e12070a0508b891c7340a350a08736c617368696e6712290a2708b891c7341220c8ccf341e6e695e7e1cb0ce4bf347eea0cc16947d8b4e934ec400b57c59d6f860a380a0b61746f6d69635f7377617012290a2708b891c734122042d4ecc9468f71a70288a95d46564bfcaf2c9f811051dcc5593dbef152976b010a110a0662726964676512070a0508b891c7340a300a0364657812290a2708b891c73412201773be443c27f61075cecdc050ce22eb4990c54679089e90afdc4e0e88182a230a2f0a02736312290a2708b891c7341220df7a0484b7244f76861b1642cfb7a61d923794bd2e076c8dbd05fc4ee29f3a670a330a06746f6b656e7312290a2708b891c734122064958c2f76fec1fa5d1828296e51264c259fa264f499724795a740f48fc4731b0a320a057374616b6512290a2708b891c734122015d2c302143bdf029d58fe381cc3b54cedf77ecb8834dfc5dc3e1555d68f19ab0a330a06706172616d7312290a2708b891c734122050abddcb7c115123a5a4247613ab39e6ba935a3d4f4b9123c4fedfa0895c040a0a300a0361636312290a2708b891c734122079fb5aecc4a9b87e56231103affa5e515a1bdf3d0366490a73e087980b7f1f260a0e0a0376616c12070a0508b891c7340a300a0369626312290a2708b891c7341220e09159530585455058cf1785f411ea44230f39334e6e0f6a3c54dbf069df2b620a300a03676f7612290a2708b891c7341220db85ddd37470983b14186e975a175dfb0bf301b43de685ced0aef18d28b4e0420a320a05706169727312290a2708b891c7341220a78b556bc9e73d86b4c63ceaf146db71b12ac80e4c10dd0ce6eb09c99b0c7cfe0a360a0974696d655f6c6f636b12290a2708b891c73412204775dbe01d41cab018c21ba5c2af94720e4d7119baf693670e70a40ba2a52143").unwrap();
        // Bytest1 is the inputs of exploit tx 0xebf83628ba893d35b496121fb8201666b8e09f3cbadf0e269162baa72efe3b8b
        let bytest2 = hex::decode("00000000000000000000000000000000000000000000000000000000000005086962630000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e00000100380200000000010dd85c0000000000000000000000000000000000000000000000000000000000000093000000000000000000000000000000000000000000000000000000000000000000f870a0424e4200000000000000000000000000000000000000000000000000000000009400000000000000000000000000000000000000008ad3c21bcecceda100000094489a8756c18c0b8b24ec2a2b9ff3d4d447f79bec94489a8756c18c0b8b24ec2a2b9ff3d4d447f79bec846553f10072cda827a83531ca0fd7ac917a6b65649719aab0836722caafe0603147a523180a8d020a066961766c3a76120e00000100380200000000010dd85c1af201f0010aed010a2b0802100318b091c73422200c10f902d266c238a4ca9e26fa9bc36483cd3ebee4e263012f5e7f40c22ee4d20a4d0801100218b091c7342220e4fd47bffd1c06e67edad92b2bf9ca63631978676288a2aa99f95c459436ef632a20da657c1ffb86c684eb3e265361ef0fa4f9dfa670b45f9f91c5eb6ad84b21a4d112001a370a0e0000010038020000000000000002122011056c6919f02d966991c10721684a8d1542e44003f9ffb47032c18995d4ac7f18b091c7341a340a0e00000100380200000000010dd85c12202c3a561458f8527b002b5ec3cab2d308662798d6245d4588a4e6a80ebdfe30ac18010ad4050a0a6d756c746973746f726512036962631ac005be050abb050a110a066f7261636c6512070a0508b891c7340a0f0a046d61696e12070a0508b891c7340a350a08736c617368696e6712290a2708b891c7341220c8ccf341e6e695e7e1cb0ce4bf347eea0cc16947d8b4e934ec400b57c59d6f860a380a0b61746f6d69635f7377617012290a2708b891c734122042d4ecc9468f71a70288a95d46564bfcaf2c9f811051dcc5593dbef152976b010a110a0662726964676512070a0508b891c7340a300a0364657812290a2708b891c73412201773be443c27f61075cecdc050ce22eb4990c54679089e90afdc4e0e88182a230a2f0a02736312290a2708b891c7341220df7a0484b7244f76861b1642cfb7a61d923794bd2e076c8dbd05fc4ee29f3a670a330a06746f6b656e7312290a2708b891c734122064958c2f76fec1fa5d1828296e51264c259fa264f499724795a740f48fc4731b0a320a057374616b6512290a2708b891c734122015d2c302143bdf029d58fe381cc3b54cedf77ecb8834dfc5dc3e1555d68f19ab0a330a06706172616d7312290a2708b891c734122050abddcb7c115123a5a4247613ab39e6ba935a3d4f4b9123c4fedfa0895c040a0a300a0361636312290a2708b891c734122079fb5aecc4a9b87e56231103affa5e515a1bdf3d0366490a73e087980b7f1f260a0e0a0376616c12070a0508b891c7340a300a0369626312290a2708b891c7341220e09159530585455058cf1785f411ea44230f39334e6e0f6a3c54dbf069df2b620a300a03676f7612290a2708b891c7341220db85ddd37470983b14186e975a175dfb0bf301b43de685ced0aef18d28b4e0420a320a05706169727312290a2708b891c7341220a78b556bc9e73d86b4c63ceaf146db71b12ac80e4c10dd0ce6eb09c99b0c7cfe0a360a0974696d655f6c6f636b12290a2708b891c73412204775dbe01d41cab018c21ba5c2af94720e4d7119baf693670e70a40ba2a52143").unwrap();

        let testcases = vec![bytest1, bytest2];

        testcases.into_iter().for_each(|t| {
            let res = iavl_proof_validation_run(&Bytes::from(t.clone()), 3_000u64);
            assert_eq!(res.err(), None);

            let res = iavl_proof_validation_run_moran(&Bytes::from(t.clone()), 3_000u64);

            assert_eq!(
                res.err(),
                Some(PrecompileError::Other(String::from("invalid merkle proof")))
            );
        });

        let input = hex::decode("00000000000000000000000000000000000000000000000000000000000007306163630000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001c6163636f756e743a8a4e2eb018bdf98a8f53ec755740ffc728637a1d000000000000000000000000000000000000000000000000000000000000007b4bdc4c270a750a148a4e2eb018bdf98a8f53ec755740ffc728637a1d12110a0941544348412d3733301080f69bf321120b0a03424e4210e8baeb8d44120f0a075050432d303041108094ebdc031a26eb5ae98721031c199c92e5b0080967da99be27cf2da53317441b4a663e6d9c6caf02be1fdbdc20d7962b28152c69c314b4de5c8035253c8bc0771d9ca17b1b23a57c0c6d068b57579791cae20add070a066961766c3a61121c6163636f756e743a8a4e2eb018bdf98a8f53ec755740ffc728637a1d1ab407b2070aaf070a2d081810cdfd2b188096a82222209f223f804e2d94ac51c4321b0687397012e6d95eb9783b03bc790da631004c7c0a2d081710adb31a18f395a8222a20d2a38865de82383ccce0140513b65cec1bf2ae6cd7dfeb22eb6faadb4e26b26f0a2d081510b2990b18f395a82222208a02bbd5a695dfc772627ac8744aa9cf30ae26575bdce8c96a9a0d0999175b430a2d081410e6ff0418f395a8222a20d39619c779be909e67f23499fb74eb2c19afd7f21523401d4ccf7e917db5cd600a2d081210e3fe0118f395a8222a20a10cc73843f889d9e03a463eb135e928bb980e19734344cba0fbf4e8a4c5258b0a2c081010dd6518f395a8222a2007fd15843a2fd3f58d021b0e072a6c70742d7a3d993a922445e3491e1c14ee8e0a2c080f10cc2a18eda6a7222a20088942d7b30abd021d8e9505cc41313fad87c8c10a799f3b51018b7b2cfe4ad90a2c080d10b70d18eda6a7222a2091a37bc44d0c61e3752ddc59eb390355ab65e8a9fb453be4f0acec537f1ca14f0a2c080c10890818eda6a72222201cfc317855a06667c45812fe36efe33af05671dfe0d9b56b02662011af2e79e30a2c080b10ac0318c4b0ee212220aeb454a4b3243b6269a2fd8841dca9a951c53b30f1e27da91063dae7224402c70a2c080910e40118c4b0ee212a20441340a4de6498f861b97b3f3ad9603af055e5af51a0d96fff2ae28e3c5c6c9a0a2c0808108d0118c4b0ee212220ae32ea4b9ab7b53571da320e2815fd8b2c278124961cca4a1849a799842424450a2b0807104d18c4b0ee212220e2804c9b7f045ec0b4ab20920a937b82fda8b7a9ddd12b21637335b915cfda550a2b0806102418a5f4c7192a20ec85f22addedfc82c771af5b4c77544b7c1d7c5bbac33f2712dfba1045ebdbd00a2b0805101118a5f4c7192a2071ade34dcc447a0ba8adc603080633d15c06f3525830c86ebce35eca0a4921fc0a2b0804100c18a5f4c7192a205190bce93993e65b266a3417ed511df8897a812cb4b62569e5afcfbec10b69cd0a2b0803100618a5f4c7192220b76c6884f1d412ac10bfb3987fb7d26f0330b2a85539509ebc5c6bdec2f95d520a2b0802100418a5f4c71922206a285b4a4f9d1c687bbafa1f3649b6a6e32b1a85dd0402421210683e846cf0020a2b0801100218a5f4c7192220033b3f7c6dcb258b6e55545e7a4f51539447cd595eb8a2e373ba0015502da1051a450a1c6163636f756e743a8a4e2eb018bdf98a8f53ec755740ffc728637a1d12201a272295e94cf1d8090bdb019dde48e9dab026ad2c3e43aaa7e61cc954a9245d18a5f4c7190ab6040a0a6d756c746973746f726512036163631aa204a0040a9d040a300a0364657812290a27088496a822122038fc49f49648fec62acc434151a51eaa378c1b20a730a749548e36f1529422500a300a03676f7612290a27088496a8221220a78ce489bdf08b9ee869c184876e1623dc38b3e64a5cf1a0005f97976c64deac0a380a0b61746f6d69635f7377617012290a27088496a8221220544c2fa38f61e10a39ec00b3e724d5834761268bb455cdbf5843bcf1531f8fbc0a300a0376616c12290a27088496a82212201f71082c9f6f45fb456b2c00b41e50d2f662f2dfec3cb6965f19d214bf02f3980a0f0a046d61696e12070a05088496a8220a320a057374616b6512290a27088496a82212200dd467343c718f240e50b4feac42970fc8c1c69a018be955f9c27913ac1f8b3c0a300a0361636312290a27088496a8221220270c19ccc9c40c5176b3dfbd8af734c97a307e0dbd8df9e286dcd5d709f973ed0a330a06746f6b656e7312290a27088496a8221220c4f96eedf50c83964de9df013afec2e545012d92528b643a5166c828774187b60a320a05706169727312290a27088496a8221220351c55cfda84596ecd22ebc77013662aba97f81f19d9ef3d150213bb07c823060a360a0974696d655f6c6f636b12290a27088496a8221220e7adf5bd30ce022decf0e9341bf05c464ed70cdbc97423bd2bab8f3571e5179b0a330a06706172616d7312290a27088496a822122042a9dfc356ca435db131eb41fb1975c8482f2434537918665e530b0b4633b5f9").unwrap();
        let res = iavl_proof_validation_run_moran(&Bytes::from(input), 3_000u64);

        assert_eq!(
            res.err(),
            Some(PrecompileError::Other(String::from("invalid merkle proof")))
        );
    }

    #[test]
    fn test_iavl_proof_validation_run_valid_proof_moran() {
        let input = hex::decode("00000000000000000000000000000000000000000000000000000000000005086962630000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e00000100380200000000010dd9ac0000000000000000000000000000000000000000000000000000000000000093000000000000000000000000000000000000000000000000000000000000000000f870a0424e4200000000000000000000000000000000000000000000000000000000009400000000000000000000000000000000000000008ad3c21bcecceda100000094489a8756c18c0b8b24ec2a2b9ff3d4d447f79bec94489a8756c18c0b8b24ec2a2b9ff3d4d447f79bec846553f10072cda827a83531ca0fd7ac917a6b65649719aab0836722caafe0603147a523180a8d020a066961766c3a76120e00000100380200000000010dd9ac1af201f0010aed010a2b0802100318b091c73422200c10f902d266c238a4ca9e26fa9bc36483cd3ebee4e263012f5e7f40c22ee4d20a4d0801100218b091c7342220e4fd47bffd1c06e67edad92b2bf9ca63631978676288a2aa99f95c459436ef632a20121a1f9c4eca726c725796c5375fc4158986ced08e498dc8268ef94d8ed1891612001a370a0e0000010038020000000000000002122011056c6919f02d966991c10721684a8d1542e44003f9ffb47032c18995d4ac7f18b091c7341a340a0e00000100380200000000010dd9ac12202c3a561458f8527b002b5ec3cab2d308662798d6245d4588a4e6a80ebdfe30ac18010ad4050a0a6d756c746973746f726512036962631ac005be050abb050a110a066f7261636c6512070a0508b891c7340a0f0a046d61696e12070a0508b891c7340a350a08736c617368696e6712290a2708b891c7341220c8ccf341e6e695e7e1cb0ce4bf347eea0cc16947d8b4e934ec400b57c59d6f860a380a0b61746f6d69635f7377617012290a2708b891c734122042d4ecc9468f71a70288a95d46564bfcaf2c9f811051dcc5593dbef152976b010a110a0662726964676512070a0508b891c7340a300a0364657812290a2708b891c73412201773be443c27f61075cecdc050ce22eb4990c54679089e90afdc4e0e88182a230a2f0a02736312290a2708b891c7341220df7a0484b7244f76861b1642cfb7a61d923794bd2e076c8dbd05fc4ee29f3a670a330a06746f6b656e7312290a2708b891c734122064958c2f76fec1fa5d1828296e51264c259fa264f499724795a740f48fc4731b0a320a057374616b6512290a2708b891c734122015d2c302143bdf029d58fe381cc3b54cedf77ecb8834dfc5dc3e1555d68f19ab0a330a06706172616d7312290a2708b891c734122050abddcb7c115123a5a4247613ab39e6ba935a3d4f4b9123c4fedfa0895c040a0a300a0361636312290a2708b891c734122079fb5aecc4a9b87e56231103affa5e515a1bdf3d0366490a73e087980b7f1f260a0e0a0376616c12070a0508b891c7340a300a0369626312290a2708b891c7341220e09159530585455058cf1785f411ea44230f39334e6e0f6a3c54dbf069df2b620a300a03676f7612290a2708b891c7341220db85ddd37470983b14186e975a175dfb0bf301b43de685ced0aef18d28b4e0420a320a05706169727312290a2708b891c7341220a78b556bc9e73d86b4c63ceaf146db71b12ac80e4c10dd0ce6eb09c99b0c7cfe0a360a0974696d655f6c6f636b12290a2708b891c73412204775dbe01d41cab018c21ba5c2af94720e4d7119baf693670e70a40ba2a52143").unwrap();
        let res = iavl_proof_validation_run(&Bytes::from(input), 3_000u64).unwrap();

        let gas = res.0;
        assert_eq!(gas, 3_000u64);

        let res = hex::encode(res.1);
        assert_eq!(
            res,
            "0000000000000000000000000000000000000000000000000000000000000001"
        )
    }

    #[test]
    fn test_iavl_proof_validation_run_valid_proof_plank() {
        let input = hex::decode("000000000000000000000000000000000000000000000000000000000000015b6962630000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000477696e640000000000000000000000000000000000000000000000000000000000000005626c6f7773ae6d1123fc362b3297bfb19c9f9fabbcbd1e2555b923dead261905b8a2ff6db60a300a0a69637332333a6961766c120477696e641a1c0a1a0a0477696e641205626c6f77731a0b0801180120012a030002040a9d010a0c69637332333a73696d706c6512036962631a87010a84010a036962631220141acb8632cfb808f293f2649cb9aabaca74fc18640900ffd0d48e2994b2a1521a090801180120012a0100222708011201011a205f0ba08283de309300409486e978a3ea59d82bccc838b07c7d39bd87c16a5034222708011201011a20455b81ef5591150bd24d3e57a769f65518b16de93487f0fab02271b3d69e2852").unwrap();
        let res = iavl_proof_validation_run_planck(&Bytes::from(input), 3_000u64).unwrap();

        let gas = res.0;
        assert_eq!(gas, 3_000u64);

        let res = hex::encode(res.1);
        assert_eq!(
            res,
            "0000000000000000000000000000000000000000000000000000000000000001"
        )
    }

    #[test]
    fn test_iavl_proof_validation_run_valid_proof_plato() {
        let input = hex::decode("000000000000000000000000000000000000000000000000000000000000015b6962630000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000477696e640000000000000000000000000000000000000000000000000000000000000005626c6f7773ae6d1123fc362b3297bfb19c9f9fabbcbd1e2555b923dead261905b8a2ff6db60a300a0a69637332333a6961766c120477696e641a1c0a1a0a0477696e641205626c6f77731a0b0801180120012a030002040a9d010a0c69637332333a73696d706c6512036962631a87010a84010a036962631220141acb8632cfb808f293f2649cb9aabaca74fc18640900ffd0d48e2994b2a1521a090801180120012a0100222708011201011a205f0ba08283de309300409486e978a3ea59d82bccc838b07c7d39bd87c16a5034222708011201011a20455b81ef5591150bd24d3e57a769f65518b16de93487f0fab02271b3d69e2852").unwrap();
        let res = iavl_proof_validation_run_plato(&Bytes::from(input), 3_000u64).unwrap();

        let gas = res.0;
        assert_eq!(gas, 3_000u64);

        let res = hex::encode(res.1);
        assert_eq!(
            res,
            "0000000000000000000000000000000000000000000000000000000000000001"
        )
    }
}
