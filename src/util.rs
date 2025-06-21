use crate::{
    EspError,
    proto_data::RpcId,
    rpc::{Rpc, setup_rpc},
};

/// Utility function to write an arbitrary message that doesn't have a payload.
/// Not exposed publically, so we have control over which messages meet this criteria.
pub(crate) fn write_empty_msg<W>(
    buf: &mut [u8],
    mut write: W,
    uid: u32,
    rpc_id: RpcId,
) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    let rpc = Rpc::new_req(rpc_id, uid);

    let frame_len = setup_rpc(buf, &rpc, &[]);
    write(&buf[..frame_len])?;

    Ok(())
}
