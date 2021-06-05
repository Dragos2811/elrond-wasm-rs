#![no_std]

mod action;
mod user_role;

use action::{Action, ActionFullInfo, PerformActionResult};
use user_role::UserRole;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const TOKEN_NUM_DECIMALS: usize = 0; //18
const INITIAL_SUPPLY: u32 = 10000u32;
const QUORUM: u32 = 6666u32;
const ESDT_ISSUE_COST: u64 = 5000000000000000000u64;

/// Multi-signature smart contract implementation.
/// Acts like a wallet that needs multiple signers for any action performed.
/// See the readme file for more detailed documentation.
#[elrond_wasm_derive::contract]
pub trait Multisig {
	/// Minimum number of signatures needed to perform any action.


	#[view(getQuorum)]
	#[storage_mapper("quorum")]
	fn quorum(&self) -> SingleValueMapper<Self::Storage, Self::BigUint>;

	#[storage_mapper("user")]
	fn user_mapper(&self) -> UserMapper<Self::Storage>;

	#[storage_get("user_role")]
	fn get_user_id_to_role(&self, user_id: usize) -> UserRole;

	#[storage_set("user_role")]
	fn set_user_id_to_role(&self, user_id: usize, user_role: UserRole);

	/// Denormalized board member count.
	/// It is kept in sync with the user list by the contract.
	#[view(getNumBoardMembers)]
	#[storage_mapper("num_board_members")]
	fn num_board_members(&self) -> SingleValueMapper<Self::Storage, usize>;

	/// Denormalized proposer count.
	/// It is kept in sync with the user list by the contract.
	#[view(getNumProposers)]
	#[storage_mapper("num_proposers")]
	fn num_proposers(&self) -> SingleValueMapper<Self::Storage, usize>;

	#[storage_mapper("action_data")]
	fn action_mapper(&self) -> VecMapper<Self::Storage, Action<Self::BigUint>>;

	/// The index of the last proposed action.
	/// 0 means that no action was ever proposed yet.
	#[view(getActionLastIndex)]
	fn get_action_last_index(&self) -> usize {
		self.action_mapper().len()
	}

	/// Serialized action data of an action with index.
	#[view(getActionData)]
	fn get_action_data(&self, action_id: usize) -> Action<Self::BigUint> {
		self.action_mapper().get(action_id)
	}

	#[storage_mapper("action_signers")]
	fn action_signers(&self, action_id: usize) -> SingleValueMapper<Self::Storage, Vec<(usize, Self::BigUint)>>;

	#[view(getCumulatedShares)]
	fn action_cumulated_shares(&self, action_id: usize) -> Self::BigUint {
		let mut total_shares = Self::BigUint::zero();
		for (_, shares) in self.action_signers(action_id).get() {
			total_shares += shares;
		}
		total_shares
	}

	#[view(getTokenIdentifier)]
	#[storage_mapper("esdt_token_id")]
	fn esdt_token_id(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

	#[view(getTokenSupply)]
	#[storage_mapper("esdt_token_supply")]
	fn esdt_token_supply(&self) -> SingleValueMapper<Self::Storage, Self::BigUint>;

	#[init]
	fn init(&self, #[var_args] board: VarArgs<Address>) -> SCResult<()> {
		require!(
			!board.is_empty(),
			"board cannot be empty on init, no-one would be able to propose"
		);
		self.quorum().set(&Self::BigUint::from(QUORUM));

		let mut duplicates = false;
		self.user_mapper()
			.get_or_create_users(board.as_slice(), |user_id, new_user| {
				if !new_user {
					duplicates = true;
				}
				self.set_user_id_to_role(user_id, UserRole::BoardMember);
			});
		require!(!duplicates, "duplicate board member");
		self.num_board_members().set(&board.len());

		Ok(())
	}

	/// Allows the contract to receive funds even if it is marked as unpayable in the protocol.
	#[payable("*")]
	#[endpoint]
	fn deposit(&self) {}

	fn propose_action(
		&self, 
		action: Action<Self::BigUint>,
		token_identifier: TokenIdentifier,
    sc_shares: Self::BigUint,
	) -> SCResult<usize> {

		require!(
			!self.esdt_token_id().is_empty(),
			"Cannot propose actions until SC token is not set"
		);

		let caller_address = self.blockchain().get_caller();
		let caller_id = self.user_mapper().get_user_id(&caller_address);
		let caller_role = self.get_user_id_to_role(caller_id);
		require!(
			caller_role.can_propose(),
			"only board members and proposers can propose"
		);

		require!(
			token_identifier == self.esdt_token_id().get(),
			"Wrong esdt"
		);

		require!(
			sc_shares > 0,
			"Member/proposer has no shares"
		);

		let action_id = self.action_mapper().push(&action);
		if caller_role.can_sign() {
			// also sign
			// since the action is newly created, the caller can be the only signer
			self.action_signers(action_id).set(&[(caller_id, sc_shares)].to_vec());
		}

		Ok(action_id)
	}

	/// Iterates through all actions and retrieves those that are still pending.
	/// Serialized full action data:
	/// - the action id
	/// - the serialized action data
	/// - (number of signers followed by) list of signer addresses.
	#[view(getPendingActionFullInfo)]
	fn get_pending_action_full_info(&self) -> MultiResultVec<ActionFullInfo<Self::BigUint>> {
		let mut result = Vec::new();
		let action_last_index = self.get_action_last_index();
		let action_mapper = self.action_mapper();
		for action_id in 1..=action_last_index {
			let action_data = action_mapper.get(action_id);
			if action_data.is_pending() {
				result.push(ActionFullInfo {
					action_id,
					action_data,
					signers: self.get_action_signers(action_id),
				});
			}
		}
		result.into()
	}

	/// Initiates board member addition process.
	/// Can also be used to promote a proposer to board member.
	#[endpoint(proposeAddBoardMember)]
	#[payable("*")]
	fn propose_add_board_member(&self, 
		board_member_address: Address,
		#[payment_token] token_identifier: TokenIdentifier,
    #[payment] sc_shares: Self::BigUint,
	) -> SCResult<usize> {
		self.propose_action(Action::AddBoardMember(board_member_address), token_identifier, sc_shares)
	}

	/// Initiates proposer addition process..
	/// Can also be used to demote a board member to proposer.
	#[endpoint(proposeAddProposer)]
	#[payable("*")]
	fn propose_add_proposer(&self, 
		proposer_address: Address,
		#[payment_token] token_identifier: TokenIdentifier,
    #[payment] sc_shares: Self::BigUint,
	) -> SCResult<usize> {
		self.propose_action(Action::AddProposer(proposer_address), token_identifier, sc_shares)
	}

	/// Removes user regardless of whether it is a board member or proposer.
	#[endpoint(proposeRemoveUser)]
	#[payable("*")]
	fn propose_remove_user(&self, 
		user_address: Address,
		#[payment_token] token_identifier: TokenIdentifier,
    #[payment] sc_shares: Self::BigUint,
	) -> SCResult<usize> {
		self.propose_action(Action::RemoveUser(user_address), token_identifier, sc_shares)
	}

	#[endpoint(proposeChangeQuorum)]
	#[payable("*")]
	fn propose_change_quorum(&self, 
		new_quorum: u32,
		#[payment_token] token_identifier: TokenIdentifier,
    #[payment] sc_shares: Self::BigUint,
	) -> SCResult<usize> {
		self.propose_action(Action::ChangeQuorum(new_quorum), token_identifier, sc_shares)
	}

	#[endpoint(proposeSendEgld)]
	#[payable("*")]
	fn propose_send_egld(
		&self,
		to: Address,
		amount: Self::BigUint,
		#[var_args] opt_data: OptionalArg<BoxedBytes>,
		#[payment_token] token_identifier: TokenIdentifier,
    #[payment] sc_shares: Self::BigUint,
	) -> SCResult<usize> {
		let data = match opt_data {
			OptionalArg::Some(data) => data,
			OptionalArg::None => BoxedBytes::empty(),
		};
		self.propose_action(Action::SendEgld { to, amount, data }, token_identifier, sc_shares)
	}

	#[endpoint(proposeSCDeploy)]
	#[payable("*")]
	fn propose_sc_deploy(
		&self,
		amount: Self::BigUint,
		code: BoxedBytes,
		upgradeable: bool,
		payable: bool,
		readable: bool,
		#[var_args] arguments: VarArgs<BoxedBytes>,
		#[payment_token] token_identifier: TokenIdentifier,
    #[payment] sc_shares: Self::BigUint,
	) -> SCResult<usize> {
		let mut code_metadata = CodeMetadata::DEFAULT;
		if upgradeable {
			code_metadata |= CodeMetadata::UPGRADEABLE;
		}
		if payable {
			code_metadata |= CodeMetadata::PAYABLE;
		}
		if readable {
			code_metadata |= CodeMetadata::READABLE;
		}
		self.propose_action(Action::SCDeploy {
			amount,
			code,
			code_metadata,
			arguments: arguments.into_vec(),
		}, token_identifier, sc_shares)
	}

	/// To be used not only for smart contract calls,
	/// but also for ESDT calls or any protocol built-in function.
	#[endpoint(proposeSCCall)]
	#[payable("*")]
	fn propose_sc_call(
		&self,
		to: Address,
		egld_payment: Self::BigUint,
		endpoint_name: BoxedBytes,
		#[var_args] arguments: VarArgs<BoxedBytes>,
		#[payment_token] token_identifier: TokenIdentifier,
    #[payment] sc_shares: Self::BigUint,
	) -> SCResult<usize> {
		self.propose_action(Action::SCCall {
			to,
			egld_payment,
			endpoint_name,
			arguments: arguments.into_vec(),
		}, token_identifier, sc_shares)
	}

	#[payable("EGLD")]
	#[endpoint(issueScToken)]
	fn issue_esdt(
		&self,
		token_display_name: BoxedBytes,
		token_ticker: BoxedBytes,
		#[payment] issue_cost: Self::BigUint,
	) -> SCResult<AsyncCall<Self::SendApi>> { 
		only_owner!(self, "only owner may call this function");

		require!(
			self.esdt_token_id().is_empty(),
			"SC token was already issued"
		);
		require!(
			issue_cost >= Self::BigUint::from(ESDT_ISSUE_COST),
			"Insufficient funds for issuing SC token"
		);
		let initial_supply = Self::BigUint::from(INITIAL_SUPPLY);
		let issue_cost = Self::BigUint::from(ESDT_ISSUE_COST);
		let caller_address = self.blockchain().get_caller();
		let owner = self.user_mapper().get_user_address_unchecked(1);

		require!(
			caller_address == owner,
			"Only owner can issue the SC token"
		);

		self.issue_started_event(&owner, token_ticker.as_slice(), &initial_supply);

		Ok(ESDTSystemSmartContractProxy::new_proxy_obj(self.send())
			.issue_fungible(
				issue_cost,
				&token_display_name,
				&token_ticker,
				&initial_supply,
				FungibleTokenProperties {
					num_decimals: TOKEN_NUM_DECIMALS,
					can_freeze: false,
					can_wipe: false,
					can_pause: false,
					can_mint: false,
					can_burn: false,
					can_change_owner: false,
					can_upgrade: true,
					can_add_special_roles: false,
				},
			)
			.async_call()
			.with_callback(self.callbacks().esdt_issue_callback(&owner)))
	}

	#[callback]
	fn esdt_issue_callback(&self,
		caller: &Address,
    #[payment_token] token_identifier: TokenIdentifier,
    #[payment] returned_tokens: Self::BigUint,
		#[call_result] result: AsyncCallResult<()>,
	) {
		// callback is called with ESDTTransfer of the newly issued token, with the amount requested,
		// so we can get the token identifier and amount from the call data
		match result {
			AsyncCallResult::Ok(()) => {
				self.issue_success_event(&caller, &token_identifier, &returned_tokens);
				self.esdt_token_supply().set(&returned_tokens);
				self.esdt_token_id().set(&token_identifier);
				if token_identifier.is_esdt() && returned_tokens > 0 {
					let _ = self.send().direct(
					&caller,
					&self.esdt_token_id().get(),
					&self.esdt_token_supply().get(),
					b"",
				);
				}
			},
			AsyncCallResult::Err(message) => {
				self.issue_failure_event(&caller, message.err_msg.as_slice());

				// if token_identifier.is_egld() && returned_tokens > 0 {
				// 	self.send().direct_egld(caller, &returned_tokens, &[]);
				// }
			},
		}
	}

	/// Returns `true` (`1`) if the user has signed the action.
	/// Does not check whether or not the user is still a board member and the signature valid.
	#[view]
	fn signed(&self, user: Address, action_id: usize) -> bool {
		let user_id = self.user_mapper().get_user_id(&user);
		if user_id == 0 {
			false
		} else {
			let signer_ids = self.action_signers(action_id).get();

			let mut contains = false;
			for (user, _) in signer_ids {
				if user == user_id {
					contains = true;
					break;
				}
			}
			contains
		}
	}

	/// Indicates user rights.
	/// `0` = no rights,
	/// `1` = can propose, but not sign,
	/// `2` = can propose and sign.
	#[view(userRole)]
	fn user_role(&self, user: Address) -> UserRole {
		let user_id = self.user_mapper().get_user_id(&user);
		if user_id == 0 {
			UserRole::None
		} else {
			self.get_user_id_to_role(user_id)
		}
	}

	/// Lists all users that can sign actions.
	#[view(getAllBoardMembers)]
	fn get_all_board_members(&self) -> MultiResultVec<Address> {
		self.get_all_users_with_role(UserRole::BoardMember)
	}

	/// Lists all proposers that are not board members.
	#[view(getAllProposers)]
	fn get_all_proposers(&self) -> MultiResultVec<Address> {
		self.get_all_users_with_role(UserRole::Proposer)
	}

	fn get_all_users_with_role(&self, role: UserRole) -> MultiResultVec<Address> {
		let mut result = Vec::new();
		let num_users = self.user_mapper().get_user_count();
		for user_id in 1..=num_users {
			if self.get_user_id_to_role(user_id) == role {
				if let Some(address) = self.user_mapper().get_user_address(user_id) {
					result.push(address);
				}
			}
		}
		result.into()
	}

	/// Used by board members to sign actions.
	#[endpoint]
	#[payable("*")]
	fn sign(
		&self, 
		action_id: usize, 
		#[payment_token] token_identifier: TokenIdentifier,
    #[payment] sc_shares: Self::BigUint,
	) -> SCResult<()> {
		require!(
			!self.action_mapper().item_is_empty_unchecked(action_id),
			"action does not exist"
		);

		require!(
			token_identifier == self.esdt_token_id().get(),
			"Wrong esdt"
		);
		
		require!(
			sc_shares > 0,
			"Member/proposer has no shares"
		);

		let caller_address = self.blockchain().get_caller();
		let caller_id = self.user_mapper().get_user_id(&caller_address);
		let caller_role = self.get_user_id_to_role(caller_id);
		require!(caller_role.can_sign(), "only board members can sign");

		let shares = sc_shares / self.esdt_token_supply().get();
		let mut contains = 0;
		for (i, (user_id, _)) in self.action_signers(action_id).get().iter().enumerate() {
			if caller_id == *user_id {
				contains = i;
				break;
			}
		}

		self.action_signers(action_id).update(|signer_ids| {
			if contains == 0 {
				signer_ids.push((caller_id, shares));
			}
			else {
				signer_ids[contains].1 += shares;	
			}
		});

		Ok(())
	}

	/// Board members can withdraw their signatures if they no longer desire for the action to be executed.
	/// Actions that are left with no valid signatures can be then deleted to free up storage.
	#[endpoint]
	fn unsign(&self, action_id: usize) -> SCResult<()> {
		require!(
			!self.action_mapper().item_is_empty_unchecked(action_id),
			"action does not exist"
		);

		let caller_address = self.blockchain().get_caller();
		let caller_id = self.user_mapper().get_user_id(&caller_address);
		let caller_role = self.get_user_id_to_role(caller_id);
		require!(caller_role.can_sign(), "only board members can un-sign");

		self.action_signers(action_id).update(|signer_ids| {
			if let Some(signer_pos) = signer_ids
				.iter()
				.position(|(signer_id, _)| *signer_id == caller_id)
			{
				// since we don't care about the order,
				// it is ok to call swap_remove, which is O(1)
				let _ = self.send().direct(
					&caller_address,
					&self.esdt_token_id().get(),
					&signer_ids[signer_pos].1,
					b"",
				);
				signer_ids.swap_remove(signer_pos);
			}
		});

		Ok(())
	}

	/// Can be used to:
	/// - create new user (board member / proposer)
	/// - remove user (board member / proposer)
	/// - reactivate removed user
	/// - convert between board member and proposer
	/// Will keep the board size and proposer count in sync.
	fn change_user_role(&self, user_address: Address, new_role: UserRole) {
		let user_id = self.user_mapper().get_or_create_user(&user_address);
		let old_role = if user_id == 0 {
			UserRole::None
		} else {
			self.get_user_id_to_role(user_id)
		};
		self.set_user_id_to_role(user_id, new_role);

		// update board size
		#[allow(clippy::collapsible_else_if)]
		if old_role == UserRole::BoardMember {
			if new_role != UserRole::BoardMember {
				self.num_board_members().update(|value| *value -= 1);
			}
		} else {
			if new_role == UserRole::BoardMember {
				self.num_board_members().update(|value| *value += 1);
			}
		}

		// update num_proposers
		#[allow(clippy::collapsible_else_if)]
		if old_role == UserRole::Proposer {
			if new_role != UserRole::Proposer {
				self.num_proposers().update(|value| *value -= 1);
			}
		} else {
			if new_role == UserRole::Proposer {
				self.num_proposers().update(|value| *value += 1);
			}
		}
	}

	/// Gets addresses of all users who signed an action.
	/// Does not check if those users are still board members or not,
	/// so the result may contain invalid signers.
	#[view(getActionSigners)]
	fn get_action_signers(&self, action_id: usize) -> Vec<Address> {
		self.action_signers(action_id)
			.get()
			.iter()
			.map(|(signer_id, _)| self.user_mapper().get_user_address_unchecked(*signer_id))
			.collect()
	}


	/// It is possible for board members to lose their role.
	/// They are not automatically removed from all actions when doing so,
	/// therefore the contract needs to re-check every time when actions are performed.
	/// This function is used to validate the signers before performing an action.
	/// It also makes it easy to check before performing an action.
	#[view(getActionValidSignerCount)]
	fn get_action_valid_signer_count(&self, action_id: usize) -> usize {
		let signer_ids = self.action_signers(action_id).get();
		signer_ids
			.iter()
			.filter(|signer| {
				let (user_id, _) = *signer;
				let signer_role = self.get_user_id_to_role(*user_id);
				signer_role.can_sign()
			})
			.count()
	}

	/// Returns `true` (`1`) if `getActionValidSignerCount >= getQuorum`.
	#[view(quorumReached)]
	fn quorum_reached(&self, action_id: usize) -> bool {
		let quorum = self.quorum().get();
		let valid_signers_count = self.action_cumulated_shares(action_id);
		valid_signers_count >= quorum
	}

	/// Proposers and board members use this to launch signed actions.
	#[endpoint(performAction)]
	fn perform_action_endpoint(
		&self,
		action_id: usize,
	) -> SCResult<PerformActionResult<Self::SendApi>> {
		let caller_address = self.blockchain().get_caller();
		let caller_id = self.user_mapper().get_user_id(&caller_address);
		let caller_role = self.get_user_id_to_role(caller_id);
		require!(
			caller_role.can_perform_action(),
			"only board members and proposers can perform actions"
		);
		require!(
			self.quorum_reached(action_id),
			"quorum has not been reached"
		);

		self.perform_action(action_id)
	}

	fn perform_action(&self, action_id: usize) -> SCResult<PerformActionResult<Self::SendApi>> {
		let action = self.action_mapper().get(action_id);

		// clean up storage
		// happens before actual execution, because the match provides the return on each branch
		// syntax aside, the async_call_raw kills contract execution so cleanup cannot happen afterwards
		self.clear_action(action_id);

		match action {
			Action::Nothing => Ok(PerformActionResult::Nothing),
			Action::AddBoardMember(board_member_address) => {
				self.change_user_role(board_member_address, UserRole::BoardMember);
				Ok(PerformActionResult::Nothing)
			},
			Action::AddProposer(proposer_address) => {
				self.change_user_role(proposer_address, UserRole::Proposer);
				Ok(PerformActionResult::Nothing)
			},
			Action::RemoveUser(user_address) => {
				self.change_user_role(user_address, UserRole::None);
				let num_board_members = self.num_board_members().get();
				let num_proposers = self.num_proposers().get();
				require!(
					num_board_members + num_proposers > 0,
					"cannot remove all board members and proposers"
				);
				Ok(PerformActionResult::Nothing)
			},
			Action::ChangeQuorum(new_quorum) => {
				self.quorum().set(&Self::BigUint::from(new_quorum));
				Ok(PerformActionResult::Nothing)
			},
			Action::SendEgld { to, amount, data } => Ok(PerformActionResult::SendEgld(SendEgld {
				api: self.send(),
				to,
				amount,
				data,
			})),
			Action::SCDeploy {
				amount,
				code,
				code_metadata,
				arguments,
			} => {
				let gas_left = self.blockchain().get_gas_left();
				let mut arg_buffer = ArgBuffer::new();
				for arg in arguments {
					arg_buffer.push_argument_bytes(arg.as_slice());
				}
				let new_address = self.send().deploy_contract(
					gas_left,
					&amount,
					&code,
					code_metadata,
					&arg_buffer,
				);
				Ok(PerformActionResult::DeployResult(new_address))
			},
			Action::SCCall {
				to,
				egld_payment,
				endpoint_name,
				arguments,
			} => {
				let mut contract_call_raw =
					ContractCall::<Self::SendApi, ()>::new(self.send(), to, endpoint_name)
						.with_token_transfer(TokenIdentifier::egld(), egld_payment);
				for arg in arguments {
					contract_call_raw.push_argument_raw_bytes(arg.as_slice());
				}
				Ok(PerformActionResult::AsyncCall(
					contract_call_raw.async_call(),
				))
			},
		}
	}

	fn clear_action(&self, action_id: usize) {
		for (user_id, shares) in self.action_signers(action_id).get() {
			let user_address = self.user_mapper().get_user_address(user_id).unwrap();
			let _ = self.send().direct(
				&user_address,
				&self.esdt_token_id().get(),
				&shares,
				b"",
			);
		}
		self.action_mapper().clear_entry_unchecked(action_id);
		self.action_signers(action_id).clear();
	}

	/// Clears storage pertaining to an action that is no longer supposed to be executed.
	/// Any signatures that the action received must first be removed, via `unsign`.
	/// Otherwise this endpoint would be prone to abuse.
	#[endpoint(discardAction)]
	fn discard_action(&self, action_id: usize) -> SCResult<()> {
		let caller_address = self.blockchain().get_caller();
		let caller_id = self.user_mapper().get_user_id(&caller_address);
		let caller_role = self.get_user_id_to_role(caller_id);
		require!(
			caller_role.can_discard_action(),
			"only board members and proposers can discard actions"
		);
		require!(
			self.get_action_valid_signer_count(action_id) == 0,
			"cannot discard action with valid signatures"
		);

		self.clear_action(action_id);
		Ok(())
	}

	//events

	#[event("issue-started")]
	fn issue_started_event(
		&self,
		#[indexed] caller: &Address,
		#[indexed] token_ticker: &[u8],
		initial_supply: &Self::BigUint,
	);

	#[event("issue-success")]
	fn issue_success_event(
		&self,
		#[indexed] caller: &Address,
		#[indexed] token_identifier: &TokenIdentifier,
		initial_supply: &Self::BigUint,
	);

	#[event("issue-failure")]
	fn issue_failure_event(&self, #[indexed] caller: &Address, message: &[u8]);
}
