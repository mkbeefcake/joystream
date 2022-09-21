// This file is part of Substrate.

// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Autogenerated weights for referendum
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-09-13, STEPS: `10`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./scripts/../target/release/joystream-node
// benchmark
// pallet
// --pallet=referendum
// --extrinsic=*
// --chain=dev
// --steps=10
// --repeat=5
// --execution=wasm
// --template=./scripts/../devops/joystream-pallet-weight-template.hbs
// --output=.

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for referendum.
pub trait WeightInfo {
	fn on_initialize_revealing(_i: u32, ) -> Weight;
	fn on_initialize_voting() -> Weight;
	fn vote() -> Weight;
	fn reveal_vote_space_for_new_winner(_i: u32, ) -> Weight;
	fn reveal_vote_space_not_in_winners(_i: u32, ) -> Weight;
	fn reveal_vote_space_replace_last_winner(_i: u32, ) -> Weight;
	fn reveal_vote_already_existing(_i: u32, ) -> Weight;
	fn release_vote_stake() -> Weight;
}

/// Weights for referendum using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: Instance1Referendum Stage (r:1 w:1)
	// Storage: Council AnnouncementPeriodNr (r:1 w:1)
	// Storage: Council Stage (r:0 w:1)
	// Storage: Council Candidates (r:5 w:5)
	// Storage: Balances Locks (r:5 w:5)
	// Storage: System Account (r:5 w:5)
	// Storage: Council CouncilorReward (r:1 w:0)
	// Storage: Council Budget (r:1 w:1)
	// Storage: Council CouncilMembers (r:1 w:1)
	// Storage: ProposalEngine Proposals (r:1 w:0)
	// Storage: Council NextRewardPayments (r:0 w:1)
	fn on_initialize_revealing(i: u32, ) -> Weight {
		(61_133_000 as Weight)
			// Standard Error: 3_865_000
			.saturating_add((1_077_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	// Storage: Instance1Referendum Stage (r:1 w:1)
	fn on_initialize_voting() -> Weight {
		(21_393_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Instance1Referendum Stage (r:1 w:0)
	// Storage: Instance1Referendum Votes (r:1 w:1)
	// Storage: Balances Locks (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn vote() -> Weight {
		(46_555_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: Instance1Referendum Stage (r:1 w:1)
	// Storage: Instance1Referendum Votes (r:1 w:1)
	// Storage: Council Candidates (r:1 w:1)
	// Storage: Council AnnouncementPeriodNr (r:1 w:0)
	fn reveal_vote_space_for_new_winner(i: u32, ) -> Weight {
		(52_422_000 as Weight)
			// Standard Error: 129_000
			.saturating_add((898_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: Instance1Referendum Stage (r:1 w:1)
	// Storage: Instance1Referendum Votes (r:1 w:1)
	// Storage: Council Candidates (r:1 w:1)
	// Storage: Council AnnouncementPeriodNr (r:1 w:0)
	fn reveal_vote_space_not_in_winners(i: u32, ) -> Weight {
		(54_154_000 as Weight)
			// Standard Error: 101_000
			.saturating_add((676_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: Instance1Referendum Stage (r:1 w:1)
	// Storage: Instance1Referendum Votes (r:1 w:1)
	// Storage: Council Candidates (r:1 w:1)
	// Storage: Council AnnouncementPeriodNr (r:1 w:0)
	fn reveal_vote_space_replace_last_winner(i: u32, ) -> Weight {
		(51_756_000 as Weight)
			// Standard Error: 61_000
			.saturating_add((1_243_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: Instance1Referendum Stage (r:1 w:1)
	// Storage: Instance1Referendum Votes (r:1 w:1)
	// Storage: Council Candidates (r:1 w:1)
	// Storage: Council AnnouncementPeriodNr (r:1 w:0)
	fn reveal_vote_already_existing(i: u32, ) -> Weight {
		(51_792_000 as Weight)
			// Standard Error: 89_000
			.saturating_add((1_076_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: Instance1Referendum Votes (r:1 w:1)
	// Storage: Council AnnouncementPeriodNr (r:1 w:0)
	// Storage: Balances Locks (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn release_vote_stake() -> Weight {
		(52_179_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
}

// Default implementation for tests
impl WeightInfo for () {
	fn on_initialize_revealing(i: u32, ) -> Weight {
		0
	}
	fn on_initialize_voting() -> Weight {
		0
	}
	fn vote() -> Weight {
		0
	}
	fn reveal_vote_space_for_new_winner(i: u32, ) -> Weight {
		0
	}
	fn reveal_vote_space_not_in_winners(i: u32, ) -> Weight {
		0
	}
	fn reveal_vote_space_replace_last_winner(i: u32, ) -> Weight {
		0
	}
	fn reveal_vote_already_existing(i: u32, ) -> Weight {
		0
	}
	fn release_vote_stake() -> Weight {
		0
	}
}
