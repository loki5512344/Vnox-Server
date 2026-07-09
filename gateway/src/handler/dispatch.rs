use anyhow::Result;
use tracing::debug;

use crate::{
    net::io,
    proto::{
        ChatMessagePayload, JoinChannelPayload, LeaveChannelPayload, PacketId, PingPayload,
        PongPayload, ReactionPayload, to_payload,
    },
};

use super::{Ctx, channel, content, direct_message, friends, guild};

pub async fn dispatch(
    ctx: &mut Ctx<'_>,
    session_id: &str,
    pid: PacketId,
    payload: &[u8],
    addr: std::net::SocketAddr,
) -> Result<()> {
    let Ctx {
        stream,
        seq,
        crypto,
        state,
    } = ctx;
    match pid {
        PacketId::Ping => {
            let ping: PingPayload = serde_json::from_slice(payload)?;
            io::send_encrypted(
                stream,
                PacketId::Pong,
                seq,
                &to_payload(&PongPayload {
                    timestamp: ping.timestamp,
                }),
                crypto,
            )
            .await?;
        }
        PacketId::JoinChannel => {
            let m: JoinChannelPayload = serde_json::from_slice(payload)?;
            channel::join(stream, seq, session_id, &m.channel_id, crypto, state).await?;
        }
        PacketId::LeaveChannel => {
            let m: LeaveChannelPayload = serde_json::from_slice(payload)?;
            channel::leave(stream, seq, session_id, &m.channel_id, crypto, state).await?;
        }
        PacketId::ChannelCreate => {
            channel::handle_channel_create(stream, seq, session_id, payload, crypto, state).await?;
        }
        PacketId::ChannelDelete => {
            channel::handle_channel_delete(stream, seq, session_id, payload, crypto, state).await?;
        }
        PacketId::ChannelList => {
            channel::handle_channel_list(stream, seq, session_id, crypto, state).await?;
        }
        PacketId::ChatMessage => {
            let m: ChatMessagePayload = serde_json::from_slice(payload)?;
            content::chat::handle(session_id, m, state).await?;
        }
        PacketId::DmStart => {
            direct_message::handle_dm_start(stream, seq, session_id, payload, crypto, state)
                .await?;
        }
        PacketId::DmMessage => {
            direct_message::handle_dm_message(stream, seq, session_id, payload, crypto, state)
                .await?;
        }
        PacketId::DmHistory => {
            direct_message::handle_dm_history(stream, seq, session_id, payload, crypto, state)
                .await?;
        }
        PacketId::DmReadAck => {
            direct_message::handle_dm_read_ack(stream, seq, session_id, payload, crypto, state)
                .await?;
        }
        PacketId::GuildCreate => {
            guild::handle_guild_create(stream, seq, session_id, payload, crypto, state).await?;
        }
        PacketId::GuildDelete => {
            guild::handle_guild_delete(stream, seq, session_id, payload, crypto, state).await?;
        }
        PacketId::GuildList => {
            guild::handle_guild_list(stream, seq, session_id, crypto, state).await?;
        }
        PacketId::GuildMemberJoin => {
            guild::handle_guild_member_join(stream, seq, session_id, payload, crypto, state)
                .await?;
        }
        PacketId::GuildMemberLeave => {
            guild::handle_guild_member_leave(stream, seq, session_id, payload, crypto, state)
                .await?;
        }
        PacketId::GuildMemberKick => {
            guild::handle_guild_member_kick(stream, seq, session_id, payload, crypto, state)
                .await?;
        }
        PacketId::RoleCreate => {
            guild::handle_role_create(stream, seq, session_id, payload, crypto, state).await?;
        }
        PacketId::RoleDelete => {
            guild::handle_role_delete(stream, seq, session_id, payload, crypto, state).await?;
        }
        PacketId::InviteCreate => {
            guild::handle_invite_create(stream, seq, session_id, payload, crypto, state).await?;
        }
        PacketId::InviteAccept => {
            guild::handle_invite_accept(stream, seq, session_id, payload, crypto, state).await?;
        }
        PacketId::InviteDelete => {
            guild::handle_invite_delete(stream, seq, session_id, payload, crypto, state).await?;
        }
        PacketId::GuildAuditLogFetch => {
            guild::handle_audit_log_fetch(stream, seq, session_id, payload, crypto, state).await?;
        }
        PacketId::GuildMemberListFetch => {
            guild::handle_member_list_fetch(stream, seq, session_id, payload, crypto, state)
                .await?;
        }
        PacketId::GuildRoleAssign => {
            guild::handle_role_assign(stream, seq, session_id, payload, crypto, state).await?;
        }
        PacketId::GuildRoleUnassign => {
            guild::handle_role_unassign(stream, seq, session_id, payload, crypto, state).await?;
        }
        PacketId::GuildRoleListFetch => {
            guild::handle_role_list_fetch(stream, seq, session_id, payload, crypto, state).await?;
        }
        PacketId::PresenceUpdate => {
            content::presence::handle_presence_update(
                stream, seq, session_id, payload, crypto, state,
            )
            .await?;
        }
        PacketId::PresenceSync => {
            content::presence::handle_presence_sync(stream, seq, session_id, crypto, state).await?;
        }
        PacketId::FriendRequest => {
            friends::handle_friend_request(stream, seq, session_id, payload, crypto, state).await?;
        }
        PacketId::FriendAccept => {
            friends::handle_friend_accept(stream, seq, session_id, payload, crypto, state).await?;
        }
        PacketId::FriendDecline => {
            friends::handle_friend_decline(stream, seq, session_id, payload, crypto, state).await?;
        }
        PacketId::FriendRemove => {
            friends::handle_friend_remove(stream, seq, session_id, payload, crypto, state).await?;
        }
        PacketId::FriendList => {
            friends::handle_friend_list(stream, seq, session_id, crypto, state).await?;
        }
        PacketId::BlockUser => {
            friends::handle_block_user(stream, seq, session_id, payload, crypto, state).await?;
        }
        PacketId::UnblockUser => {
            friends::handle_unblock_user(stream, seq, session_id, payload, crypto, state).await?;
        }
        PacketId::BlockList => {
            friends::handle_block_list(stream, seq, session_id, crypto, state).await?;
        }
        PacketId::MessageReactionAdd => {
            let m: ReactionPayload = serde_json::from_slice(payload)?;
            content::reaction::handle_reaction_add(session_id, m, state).await?;
        }
        PacketId::MessageReactionRemove => {
            let m: ReactionPayload = serde_json::from_slice(payload)?;
            content::reaction::handle_reaction_remove(session_id, m, state).await?;
        }
        PacketId::MessageEdit => {
            content::message_edit::handle_message_edit(session_id, payload, state).await?;
        }
        PacketId::MessageDelete => {
            content::message_edit::handle_message_delete(session_id, payload, state).await?;
        }
        PacketId::TypingStart => {
            content::handle_typing_start(session_id, payload, state).await?;
        }
        PacketId::ReadReceipt => {
            content::handle_read_receipt(stream, seq, session_id, payload, crypto, state).await?;
        }
        PacketId::Disconnect => debug!("{addr} DISCONNECT"),
        other => debug!("{addr} unhandled {:?}", other),
    }
    Ok(())
}
