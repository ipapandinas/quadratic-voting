## Quadratic Voting pallet for a decentralized voting platform

A quadratic voting systems allows users to vote on an egalitarian basis with a distribution of preferences for decision-making.

### Abstract

This pallet equips users with the tools required to build a decentralized quadratic voting platform. A well-known account can register to the platform and then create several proposals with customize voting process for each one based on their needs. Then other allowed registered voters vote 'aye' or 'nay'. A proposal is successfully approved when the 'aye' ratio (first item) is greater than 0.5 compare with the half of total number of votes (second item / 2).

### Customization aspects

#### Offchain Data

The creator provides an offchain data string that can be an IPFS CID hash that points to a JSON file, a plain text, a small JSON string, or a link to either a static or a dynamic file. The maximum length allowed for this offchain data string can be set in the configuration using `ProposalOffchainDataLimit`.

Here is an example of metadata JSON file:

```json
{
	"title":"This is the title of the proposal",
	"description":"This is the description of the proposal",
}
```

#### Duration

Each proposal has a specific duration specified with `start_block` and `end_block`. The maximum duration allowed can be set in the configuration using `ProposalMaximumDuration`. The minimum duration allowed can be set in the configuration using `ProposalMinimumDuration`.

### Scheduling

A proposal can be scheduled adding some delay to the `start_block`. The maximum delay allowed can be set in the configuration using `ProposalDelayLimit`.

### Proposal kind: Public or Private

Proposals can be public or private when created:
- A public proposal allows any registered voter of the platform to vote for this proposal. The `account_list` refers to the ban list, a banned user cannot vote for a public proposal.
- A private proposal is a quorum voting. The creator specify the allowed voters that can vote for this proposal.

The account list limit can be set in the configuration using `AccountSizeLimit`.

### Interface

```rust
interface {
	/// Description: A registered voter can submit a new proposal by providing offchain data string that can be IPFS CID hash that points to a JSON file, a plain text, a small JSON string, or a link to either a static or a dynamic file.
	/// The proposal can be private (quorum system) or public (accessible by others registered voters).
	/// Constraint(s):
	///     - User must be a registered voter.
	///	    - If public -> private, account_list becomes an allow list instead of a ban list
	///	    - If private -> public, account_list becomes a ban list instead of an allow list
	///     - Start block must not start in the past.
	///     - End block must not be before start block.
	///     - Duration must not be too long.
	///     - Duration must not be too short.
	///     - Proposal start block must not be too far in the future.
	create_proposal(offchain_data: BoundedVec<u8, ProposalOffchainDataLimit>, kind: ProposalKind, account_list: BoundedVec<AccountId, AccountSizeLimit>, start_block: BlockNumber, end_Block: BlockNumber)

	/// Description: User can cancel a proposal that has not started yet.
	/// Constraint(s): 
	///     - User must be creator of the proposal.
	///     - Proposal must not have started.
	cancel_proposal(proposal_id: ProposalId)

	/// Description: User can close a proposal that is finished. Free call, no fee.
	/// Constraint(s): 
	///     - User must be creator of the proposal or root.
	///     - Proposal must have finished.
	close_proposal(proposal_id: ProposalId)

	/// Description: User can change the account_list for a proposal that has not started yet.
	/// Constraint(s): 
	///     - User must be creator of the proposal.
	///     - Proposal must not have started.
	set_account_list(proposal_id: ProposalId, account_list: BoundedVec<AccountId, AccountSizeLimit>)

	/// Description: Register a new voter.
	/// Constraint(s): 
	///     - Root or voter only.
	register_voter(who: AccountId)

	/// Description: Unregister a registered voter. Free call, no fee. Registered voter as signer or Root.
	/// Constraint(s): 
	///     - Ensure correct signer.
	unregister_voter(who: AccountId)

	/// Description: Vote for an in progress proposal with a given weight. A private proposal is closed if majority is reached.
	/// Constraint(s):
	///     - Ensure registered voter.
	///     - Ensure correct behavior base on account_list
	///     - Proposal must have started.
	///     - Voter must have sufficient funds to vote in a quadratic manner based on the provided weight.
	vote(proposal_id: ProposalId, aye: bool, weight: Option<u32>)

    /// Description: Unfreeze the locked amount of a vote.
	/// Constraint(s):
	///     - Ensure registered voter.
	///     - Proposal must be closed.
	///     - Voter must be a valid voter for this proposal.
	claim(proposal_id: ProposalId)
}
```

### Contraints

- A voter must be registered to interact with proposals in the platform (except for closing which is a free call).
- A proposal cannot start in the past nor finish before starting.
- A claim is available only for a closed proposal and an existing voter.
- A proposal can be cancelled or the account list can be updated if the proposal has not started yet.

### Future ideas

An unsigned tx call hash can be attached to a proposal and be executed by the root account when closing the proposal if success.

---

## [Substrate Node Template](https://github.com/substrate-developer-hub/substrate-node-template)

A fresh FRAME-based [Substrate](https://www.substrate.io/) node, ready for hacking :rocket:

### Setup

Please first check the latest information on getting starting with Substrate dependencies required to build this project [here](https://docs.substrate.io/main-docs/install/).

### Development Testing

To test while developing, without a full build (thus reduce time to results):

```sh
cargo t -p pallet-voting
cargo t -p <other crates>
```

### Build

Build the node without launching it, with `release` optimizations:

```sh
cargo b -r
```

### Run

Build and launch the node, with `release` optimizations:

```sh
cargo r -r -- --dev
```

### CLI Docs

Once the project has been built, the following command can be used to explore all CLI arguments and subcommands:

```sh
./target/release/node-template -h
```
