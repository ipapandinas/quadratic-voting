# Tests

## Unit test

1. UnregisterVoter (Clean ups)

2. Vote
    setup:
    - registered voter
    - balance minted

    checks: 
    - Ensure registered voter.
    - Proposal must exist
    - Proposal must have started.
    - Proposal must not have ended.
    - Ensure correct behavior base on account_list.
    - Ensure not identic vote.
    - Voter must have sufficient funds to vote in a quadratic manner based on the provided power.

3. Claim

## E2E test

### Quorum vote

Happy scenario: VOTE SUCCESS

**Prerequisities**

- 5 registered voters: Alice / Bob / Charlie / Dave / Eve
- Alice creates proposals
    - kind: Private
    - Account list (Allowed voters): Alice / Bob / Dave

**Ensure**

- Event NewProposal
- Storage contains Proposal
- Set account list with Charlie during the delay period
- Vote start:
    Alice 'aye'*2 => freeze 4 /
    Bob 'nay'*4 => freeze 16 /
    Eve 'error - not allowed' /
    Dave 'aye'*3 => freeze 9 /
    Bob 'nay'*3 => unfreeze 7 /
    Charlie 'aye'*2 => freeze 4 /
    Alice 'aye'*0 => unfreeze 4 /  ==> VoteDropped + Votes storage clean up
- Eve close vote => VoteCompleted ratio (13/22)
- Bob / Dave / Charlie claim