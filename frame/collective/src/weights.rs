// This file is part of Substrate.

// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Autogenerated weights for pallet_collective
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-11-07, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `bm2`, CPU: `Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/substrate
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet_collective
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./frame/collective/src/weights.rs
// --header=./HEADER-APACHE2
// --template=./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_collective.
pub trait WeightInfo {
	fn set_members(m: u32, n: u32, p: u32, ) -> Weight;
	fn execute(b: u32, m: u32, ) -> Weight;
	fn propose_execute(b: u32, m: u32, ) -> Weight;
	fn propose_proposed(b: u32, m: u32, p: u32, ) -> Weight;
	fn vote(m: u32, ) -> Weight;
	fn close_early_disapproved(m: u32, p: u32, ) -> Weight;
	fn close_early_approved(b: u32, m: u32, p: u32, ) -> Weight;
	fn close_disapproved(m: u32, p: u32, ) -> Weight;
	fn close_approved(b: u32, m: u32, p: u32, ) -> Weight;
	fn disapprove_proposal(p: u32, ) -> Weight;
}

/// Weights for pallet_collective using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: Council Members (r:1 w:1)
	// Storage: Council Proposals (r:1 w:0)
	// Storage: Council Prime (r:0 w:1)
	// Storage: Council Voting (r:100 w:100)
	/// The range of component `m` is `[0, 100]`.
	/// The range of component `n` is `[0, 100]`.
	/// The range of component `p` is `[0, 100]`.
	fn set_members(m: u32, _n: u32, p: u32, ) -> Weight {
		// Minimum execution time: 18_895 nanoseconds.
		Weight::from_ref_time(19_254_000 as u64)
			// Standard Error: 63_540
			.saturating_add(Weight::from_ref_time(5_061_801 as u64).saturating_mul(m as u64))
			// Standard Error: 63_540
			.saturating_add(Weight::from_ref_time(7_588_981 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().reads((1 as u64).saturating_mul(p as u64)))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
			.saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(p as u64)))
	}
	// Storage: Council Members (r:1 w:0)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn execute(b: u32, m: u32, ) -> Weight {
		// Minimum execution time: 24_469 nanoseconds.
		Weight::from_ref_time(23_961_134 as u64)
			// Standard Error: 43
			.saturating_add(Weight::from_ref_time(1_677 as u64).saturating_mul(b as u64))
			// Standard Error: 450
			.saturating_add(Weight::from_ref_time(18_645 as u64).saturating_mul(m as u64))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
	}
	// Storage: Council Members (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:0)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn propose_execute(b: u32, m: u32, ) -> Weight {
		// Minimum execution time: 26_476 nanoseconds.
		Weight::from_ref_time(25_829_298 as u64)
			// Standard Error: 49
			.saturating_add(Weight::from_ref_time(1_741 as u64).saturating_mul(b as u64))
			// Standard Error: 515
			.saturating_add(Weight::from_ref_time(29_436 as u64).saturating_mul(m as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
	}
	// Storage: Council Members (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:1)
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalCount (r:1 w:1)
	// Storage: Council Voting (r:0 w:1)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[2, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn propose_proposed(b: u32, m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 33_585 nanoseconds.
		Weight::from_ref_time(33_092_289 as u64)
			// Standard Error: 173
			.saturating_add(Weight::from_ref_time(4_266 as u64).saturating_mul(b as u64))
			// Standard Error: 1_812
			.saturating_add(Weight::from_ref_time(29_262 as u64).saturating_mul(m as u64))
			// Standard Error: 1_789
			.saturating_add(Weight::from_ref_time(181_285 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Voting (r:1 w:1)
	/// The range of component `m` is `[5, 100]`.
	fn vote(m: u32, ) -> Weight {
		// Minimum execution time: 36_374 nanoseconds.
		Weight::from_ref_time(38_950_243 as u64)
			// Standard Error: 2_583
			.saturating_add(Weight::from_ref_time(65_345 as u64).saturating_mul(m as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalOf (r:0 w:1)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_disapproved(m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 36_066 nanoseconds.
		Weight::from_ref_time(38_439_655 as u64)
			// Standard Error: 1_281
			.saturating_add(Weight::from_ref_time(17_045 as u64).saturating_mul(m as u64))
			// Standard Error: 1_249
			.saturating_add(Weight::from_ref_time(164_998 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:1)
	// Storage: Council Proposals (r:1 w:1)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_approved(b: u32, m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 47_753 nanoseconds.
		Weight::from_ref_time(46_507_829 as u64)
			// Standard Error: 149
			.saturating_add(Weight::from_ref_time(2_159 as u64).saturating_mul(b as u64))
			// Standard Error: 1_581
			.saturating_add(Weight::from_ref_time(37_842 as u64).saturating_mul(m as u64))
			// Standard Error: 1_541
			.saturating_add(Weight::from_ref_time(173_395 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Prime (r:1 w:0)
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalOf (r:0 w:1)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_disapproved(m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 39_416 nanoseconds.
		Weight::from_ref_time(39_610_161 as u64)
			// Standard Error: 1_231
			.saturating_add(Weight::from_ref_time(32_991 as u64).saturating_mul(m as u64))
			// Standard Error: 1_200
			.saturating_add(Weight::from_ref_time(170_773 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Prime (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:1)
	// Storage: Council Proposals (r:1 w:1)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_approved(b: u32, m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 49_840 nanoseconds.
		Weight::from_ref_time(48_542_914 as u64)
			// Standard Error: 136
			.saturating_add(Weight::from_ref_time(2_650 as u64).saturating_mul(b as u64))
			// Standard Error: 1_442
			.saturating_add(Weight::from_ref_time(37_898 as u64).saturating_mul(m as u64))
			// Standard Error: 1_406
			.saturating_add(Weight::from_ref_time(182_176 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council Voting (r:0 w:1)
	// Storage: Council ProposalOf (r:0 w:1)
	/// The range of component `p` is `[1, 100]`.
	fn disapprove_proposal(p: u32, ) -> Weight {
		// Minimum execution time: 24_199 nanoseconds.
		Weight::from_ref_time(26_869_176 as u64)
			// Standard Error: 1_609
			.saturating_add(Weight::from_ref_time(163_341 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Council Members (r:1 w:1)
	// Storage: Council Proposals (r:1 w:0)
	// Storage: Council Prime (r:0 w:1)
	// Storage: Council Voting (r:100 w:100)
	/// The range of component `m` is `[0, 100]`.
	/// The range of component `n` is `[0, 100]`.
	/// The range of component `p` is `[0, 100]`.
	fn set_members(m: u32, _n: u32, p: u32, ) -> Weight {
		// Minimum execution time: 18_895 nanoseconds.
		Weight::from_ref_time(19_254_000 as u64)
			// Standard Error: 63_540
			.saturating_add(Weight::from_ref_time(5_061_801 as u64).saturating_mul(m as u64))
			// Standard Error: 63_540
			.saturating_add(Weight::from_ref_time(7_588_981 as u64).saturating_mul(p as u64))
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().reads((1 as u64).saturating_mul(p as u64)))
			.saturating_add(RocksDbWeight::get().writes(2 as u64))
			.saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(p as u64)))
	}
	// Storage: Council Members (r:1 w:0)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn execute(b: u32, m: u32, ) -> Weight {
		// Minimum execution time: 24_469 nanoseconds.
		Weight::from_ref_time(23_961_134 as u64)
			// Standard Error: 43
			.saturating_add(Weight::from_ref_time(1_677 as u64).saturating_mul(b as u64))
			// Standard Error: 450
			.saturating_add(Weight::from_ref_time(18_645 as u64).saturating_mul(m as u64))
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
	}
	// Storage: Council Members (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:0)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn propose_execute(b: u32, m: u32, ) -> Weight {
		// Minimum execution time: 26_476 nanoseconds.
		Weight::from_ref_time(25_829_298 as u64)
			// Standard Error: 49
			.saturating_add(Weight::from_ref_time(1_741 as u64).saturating_mul(b as u64))
			// Standard Error: 515
			.saturating_add(Weight::from_ref_time(29_436 as u64).saturating_mul(m as u64))
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
	}
	// Storage: Council Members (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:1)
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalCount (r:1 w:1)
	// Storage: Council Voting (r:0 w:1)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[2, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn propose_proposed(b: u32, m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 33_585 nanoseconds.
		Weight::from_ref_time(33_092_289 as u64)
			// Standard Error: 173
			.saturating_add(Weight::from_ref_time(4_266 as u64).saturating_mul(b as u64))
			// Standard Error: 1_812
			.saturating_add(Weight::from_ref_time(29_262 as u64).saturating_mul(m as u64))
			// Standard Error: 1_789
			.saturating_add(Weight::from_ref_time(181_285 as u64).saturating_mul(p as u64))
			.saturating_add(RocksDbWeight::get().reads(4 as u64))
			.saturating_add(RocksDbWeight::get().writes(4 as u64))
	}
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Voting (r:1 w:1)
	/// The range of component `m` is `[5, 100]`.
	fn vote(m: u32, ) -> Weight {
		// Minimum execution time: 36_374 nanoseconds.
		Weight::from_ref_time(38_950_243 as u64)
			// Standard Error: 2_583
			.saturating_add(Weight::from_ref_time(65_345 as u64).saturating_mul(m as u64))
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalOf (r:0 w:1)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_disapproved(m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 36_066 nanoseconds.
		Weight::from_ref_time(38_439_655 as u64)
			// Standard Error: 1_281
			.saturating_add(Weight::from_ref_time(17_045 as u64).saturating_mul(m as u64))
			// Standard Error: 1_249
			.saturating_add(Weight::from_ref_time(164_998 as u64).saturating_mul(p as u64))
			.saturating_add(RocksDbWeight::get().reads(3 as u64))
			.saturating_add(RocksDbWeight::get().writes(3 as u64))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:1)
	// Storage: Council Proposals (r:1 w:1)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_approved(b: u32, m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 47_753 nanoseconds.
		Weight::from_ref_time(46_507_829 as u64)
			// Standard Error: 149
			.saturating_add(Weight::from_ref_time(2_159 as u64).saturating_mul(b as u64))
			// Standard Error: 1_581
			.saturating_add(Weight::from_ref_time(37_842 as u64).saturating_mul(m as u64))
			// Standard Error: 1_541
			.saturating_add(Weight::from_ref_time(173_395 as u64).saturating_mul(p as u64))
			.saturating_add(RocksDbWeight::get().reads(4 as u64))
			.saturating_add(RocksDbWeight::get().writes(3 as u64))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Prime (r:1 w:0)
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalOf (r:0 w:1)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_disapproved(m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 39_416 nanoseconds.
		Weight::from_ref_time(39_610_161 as u64)
			// Standard Error: 1_231
			.saturating_add(Weight::from_ref_time(32_991 as u64).saturating_mul(m as u64))
			// Standard Error: 1_200
			.saturating_add(Weight::from_ref_time(170_773 as u64).saturating_mul(p as u64))
			.saturating_add(RocksDbWeight::get().reads(4 as u64))
			.saturating_add(RocksDbWeight::get().writes(3 as u64))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Prime (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:1)
	// Storage: Council Proposals (r:1 w:1)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_approved(b: u32, m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 49_840 nanoseconds.
		Weight::from_ref_time(48_542_914 as u64)
			// Standard Error: 136
			.saturating_add(Weight::from_ref_time(2_650 as u64).saturating_mul(b as u64))
			// Standard Error: 1_442
			.saturating_add(Weight::from_ref_time(37_898 as u64).saturating_mul(m as u64))
			// Standard Error: 1_406
			.saturating_add(Weight::from_ref_time(182_176 as u64).saturating_mul(p as u64))
			.saturating_add(RocksDbWeight::get().reads(5 as u64))
			.saturating_add(RocksDbWeight::get().writes(3 as u64))
	}
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council Voting (r:0 w:1)
	// Storage: Council ProposalOf (r:0 w:1)
	/// The range of component `p` is `[1, 100]`.
	fn disapprove_proposal(p: u32, ) -> Weight {
		// Minimum execution time: 24_199 nanoseconds.
		Weight::from_ref_time(26_869_176 as u64)
			// Standard Error: 1_609
			.saturating_add(Weight::from_ref_time(163_341 as u64).saturating_mul(p as u64))
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(3 as u64))
	}
}
