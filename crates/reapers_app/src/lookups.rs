//! Display names for the things the rules only know by identifier.
//!
//! The rules stay language-free: they speak in enums, and these functions
//! turn an enum into the player's language. Every match is exhaustive, so a
//! new class or weapon cannot ship without its name.

use reapers_program_client::errors::ReapersProgramError;
use reapers_rules::Art;
use reapers_rules::Aura;
use reapers_rules::CaptureGrade;
use reapers_rules::ClassPerk;
use reapers_rules::Difficulty;
use reapers_rules::Feeding;
use reapers_rules::HuntRules;
use reapers_rules::Omen;
use reapers_rules::Outfit;
use reapers_rules::Rarity;
use reapers_rules::ReaperClass;
use reapers_rules::ReaperGift;
use reapers_rules::Refusal;
use reapers_rules::SoulPath;
use reapers_rules::SoulRules;
use reapers_rules::Temper;
use reapers_rules::Temperament;
use reapers_rules::WeaponKind;

use crate::chain::ChainFailure;
use crate::chain::ClusterName;
use crate::l10n::Messages;
use crate::simnet::SimnetInstruction;
use crate::simnet::SimnetRole;

pub fn class_name(words: &dyn Messages, value: ReaperClass) -> &'static str {
	match value {
		ReaperClass::Harvester => words.class_harvester(),
		ReaperClass::Ferryman => words.class_ferryman(),
		ReaperClass::Weigher => words.class_weigher(),
		ReaperClass::Chooser => words.class_chooser(),
		ReaperClass::Hollow => words.class_hollow(),
		ReaperClass::Mourner => words.class_mourner(),
	}
}

pub fn class_line(words: &dyn Messages, value: ReaperClass) -> &'static str {
	match value {
		ReaperClass::Harvester => words.class_harvester_line(),
		ReaperClass::Ferryman => words.class_ferryman_line(),
		ReaperClass::Weigher => words.class_weigher_line(),
		ReaperClass::Chooser => words.class_chooser_line(),
		ReaperClass::Hollow => words.class_hollow_line(),
		ReaperClass::Mourner => words.class_mourner_line(),
	}
}

pub fn perk_name(words: &dyn Messages, value: ClassPerk) -> &'static str {
	match value {
		ClassPerk::Cleave => words.perk_cleave(),
		ClassPerk::Steadfast => words.perk_steadfast(),
		ClassPerk::Measure => words.perk_measure(),
		ClassPerk::ChosenBlow => words.perk_chosen_blow(),
		ClassPerk::Chain => words.perk_chain(),
		ClassPerk::Toll => words.perk_toll(),
	}
}

pub fn perk_line(words: &dyn Messages, value: ClassPerk) -> &'static str {
	match value {
		ClassPerk::Cleave => words.perk_cleave_line(),
		ClassPerk::Steadfast => words.perk_steadfast_line(),
		ClassPerk::Measure => words.perk_measure_line(),
		ClassPerk::ChosenBlow => words.perk_chosen_blow_line(),
		ClassPerk::Chain => words.perk_chain_line(),
		ClassPerk::Toll => words.perk_toll_line(),
	}
}

pub fn weapon_name(words: &dyn Messages, value: WeaponKind) -> &'static str {
	match value {
		WeaponKind::Scythe => words.weapon_scythe(),
		WeaponKind::LanternPole => words.weapon_lantern_pole(),
		WeaponKind::BalanceFlail => words.weapon_balance_flail(),
		WeaponKind::Spear => words.weapon_spear(),
		WeaponKind::ChainWhip => words.weapon_chain_whip(),
		WeaponKind::Bell => words.weapon_bell(),
		WeaponKind::Greatsword => words.weapon_greatsword(),
		WeaponKind::Axe => words.weapon_axe(),
	}
}

/// What sets a heavy weapon apart, or `None` for any other.
pub fn weapon_trait(words: &dyn Messages, value: WeaponKind) -> Option<&'static str> {
	match value {
		WeaponKind::Greatsword => Some(words.weapon_greatsword_trait()),
		WeaponKind::Axe => Some(words.weapon_axe_trait()),
		_ => None,
	}
}

pub fn art_name(words: &dyn Messages, value: Art) -> &'static str {
	match value {
		Art::Cinder => words.art_cinder(),
		Art::Rime => words.art_rime(),
		Art::Gale => words.art_gale(),
	}
}

pub fn art_line(words: &dyn Messages, value: Art) -> &'static str {
	match value {
		Art::Cinder => words.art_cinder_line(),
		Art::Rime => words.art_rime_line(),
		Art::Gale => words.art_gale_line(),
	}
}

pub fn temper_name(words: &dyn Messages, value: Temper) -> &'static str {
	match value {
		Temper::Shrouded => words.temper_shrouded(),
		Temper::Sworn => words.temper_sworn(),
		Temper::Scorched => words.temper_scorched(),
		Temper::Hollowed => words.temper_hollowed(),
	}
}

/// An outfit is named after the same four states as a temper.
pub fn outfit_name(words: &dyn Messages, value: Outfit) -> &'static str {
	match value {
		Outfit::Shrouded => words.temper_shrouded(),
		Outfit::Sworn => words.temper_sworn(),
		Outfit::Scorched => words.temper_scorched(),
		Outfit::Hollowed => words.temper_hollowed(),
	}
}

pub fn outfit_line(words: &dyn Messages, value: Outfit) -> &'static str {
	match value {
		Outfit::Shrouded => words.outfit_shrouded_line(),
		Outfit::Sworn => words.outfit_sworn_line(),
		Outfit::Scorched => words.outfit_scorched_line(),
		Outfit::Hollowed => words.outfit_hollowed_line(),
	}
}

pub fn rarity_name(words: &dyn Messages, value: Rarity) -> &'static str {
	match value {
		Rarity::Common => words.rarity_common(),
		Rarity::Uncommon => words.rarity_uncommon(),
		Rarity::Rare => words.rarity_rare(),
		Rarity::Epic => words.rarity_epic(),
	}
}

pub fn feeding_name(words: &dyn Messages, value: Feeding) -> &'static str {
	match value {
		Feeding::Meagre => words.feeding_meagre(),
		Feeding::Fair => words.feeding_fair(),
		Feeding::Rich => words.feeding_rich(),
		Feeding::Feast => words.feeding_feast(),
	}
}

pub fn grade_name(words: &dyn Messages, value: CaptureGrade) -> &'static str {
	match value {
		CaptureGrade::Excellent => words.hunt_grade_excellent(),
		CaptureGrade::Great => words.hunt_grade_great(),
		CaptureGrade::Plain => words.hunt_grade_plain(),
	}
}

pub fn omen_name(words: &dyn Messages, value: Omen) -> &'static str {
	match value {
		Omen::None => words.omen_none(),
		Omen::Ashen => words.omen_ashen(),
		Omen::Gilded => words.omen_gilded(),
		Omen::Eclipsed => words.omen_eclipsed(),
	}
}

pub fn refusal_message(words: &dyn Messages, value: Refusal) -> &'static str {
	match value {
		Refusal::OathAlreadyTaken => words.refusal_oath_already_taken(),
		Refusal::OathRequired => words.refusal_oath_required(),
		Refusal::NotListed => words.refusal_not_listed(),
		Refusal::AlreadyHeld => words.refusal_already_held(),
		Refusal::NotInVault => words.refusal_not_in_vault(),
		Refusal::InsufficientFunds => words.refusal_insufficient_funds(),
		Refusal::InsufficientAnima => words.refusal_insufficient_anima(),
		Refusal::LevelTooLow => words.refusal_level_too_low(),
		Refusal::NotOwned => words.refusal_not_owned(),
		Refusal::AlreadyOwned => words.refusal_already_owned(),
		Refusal::VeilCrowded => words.refusal_veil_crowded(),
		Refusal::UnknownSighting => words.refusal_unknown_sighting(),
		Refusal::NotSurfaced => words.refusal_not_surfaced(),
		Refusal::HuntUnproven => words.refusal_hunt_unproven(),
		Refusal::HuntVersionUnsupported => words.refusal_hunt_version_unsupported(),
		Refusal::MarketUnavailable => words.refusal_market_unavailable(),
		Refusal::RiftUnproven => words.refusal_rift_unproven(),
		Refusal::NotAttuned => words.refusal_not_attuned(),
		Refusal::LocationUnusable => words.location_inaccurate(),
		Refusal::ReaperBusy => words.refusal_reaper_busy(),
		Refusal::WalletUnproven => words.refusal_wallet_unproven(),
		Refusal::ChainUnconfigured => words.refusal_chain_unconfigured(),
		Refusal::ChainUnavailable => words.common_unreachable(),
		Refusal::RosterFull => words.refusal_roster_full(),
	}
}

pub fn aura_name(words: &dyn Messages, value: Aura) -> &'static str {
	match value {
		Aura::Motes => words.studio_aura_motes(),
		Aura::Embers => words.studio_aura_embers(),
		Aura::Mist => words.studio_aura_mist(),
		Aura::Wisps => words.studio_aura_wisps(),
	}
}

pub fn difficulty_name(words: &dyn Messages, value: Difficulty) -> &'static str {
	match value {
		Difficulty::Easy => words.difficulty_easy(),
		Difficulty::Normal => words.difficulty_normal(),
		Difficulty::Hard => words.difficulty_hard(),
	}
}

pub fn simnet_instruction_name(words: &dyn Messages, value: SimnetInstruction) -> &'static str {
	match value {
		SimnetInstruction::Airdrop => words.simnet_instruction_airdrop(),
		SimnetInstruction::Buy => words.simnet_instruction_buy(),
		SimnetInstruction::Escrow => words.simnet_instruction_escrow(),
		SimnetInstruction::Burn => words.simnet_instruction_burn(),
	}
}

pub fn simnet_role_name(words: &dyn Messages, value: SimnetRole) -> &'static str {
	match value {
		SimnetRole::Wallet => words.simnet_role_wallet(),
		SimnetRole::Faucet => words.simnet_role_faucet(),
		SimnetRole::Market => words.simnet_role_market(),
		SimnetRole::Mint => words.simnet_role_mint(),
		SimnetRole::Escrow => words.simnet_role_escrow(),
		SimnetRole::Program => words.simnet_role_program(),
		SimnetRole::Keeper => words.simnet_role_keeper(),
	}
}

pub fn soul_path_name(words: &dyn Messages, value: SoulPath) -> &'static str {
	match value {
		SoulPath::Unseen => words.soul_unseen(),
		SoulPath::Stillness => words.soul_stillness(),
		SoulPath::Rhythm => words.soul_rhythm(),
	}
}

pub fn soul_path_help(words: &dyn Messages, value: SoulPath) -> &'static str {
	match value {
		SoulPath::Unseen => words.soul_unseen_help(),
		SoulPath::Stillness => words.soul_stillness_help(),
		SoulPath::Rhythm => words.soul_rhythm_help(),
	}
}

pub fn temperament_name(words: &dyn Messages, value: Temperament) -> &'static str {
	match value {
		Temperament::Patient => words.temperament_patient(),
		Temperament::Keen => words.temperament_keen(),
		Temperament::Stalwart => words.temperament_stalwart(),
		Temperament::Hungry => words.temperament_hungry(),
	}
}

pub fn temperament_help(words: &dyn Messages, value: Temperament) -> &'static str {
	match value {
		Temperament::Patient => words.temperament_patient_help(),
		Temperament::Keen => words.temperament_keen_help(),
		Temperament::Stalwart => words.temperament_stalwart_help(),
		Temperament::Hungry => words.temperament_hungry_help(),
	}
}

pub fn gift_name(words: &dyn Messages, value: ReaperGift) -> &'static str {
	match value {
		ReaperGift::Furrow => words.gift_furrow(),
		ReaperGift::Aftermath => words.gift_aftermath(),
		ReaperGift::Gathering => words.gift_gathering(),
		ReaperGift::Passage => words.gift_passage(),
		ReaperGift::Anchor => words.gift_anchor(),
		ReaperGift::ReturnLight => words.gift_return_light(),
		ReaperGift::Foreknowledge => words.gift_foreknowledge(),
		ReaperGift::Verdict => words.gift_verdict(),
		ReaperGift::Equilibrium => words.gift_equilibrium(),
		ReaperGift::FirstLight => words.gift_first_light(),
		ReaperGift::Reprisal => words.gift_reprisal(),
		ReaperGift::LastWord => words.gift_last_word(),
		ReaperGift::Hunger => words.gift_hunger(),
		ReaperGift::Echo => words.gift_echo(),
		ReaperGift::Unbroken => words.gift_unbroken(),
		ReaperGift::Hush => words.gift_hush(),
		ReaperGift::Keening => words.gift_keening(),
		ReaperGift::Refrain => words.gift_refrain(),
	}
}

pub fn gift_lore(words: &dyn Messages, value: ReaperGift) -> &'static str {
	match value {
		ReaperGift::Furrow => words.gift_furrow_lore(),
		ReaperGift::Aftermath => words.gift_aftermath_lore(),
		ReaperGift::Gathering => words.gift_gathering_lore(),
		ReaperGift::Passage => words.gift_passage_lore(),
		ReaperGift::Anchor => words.gift_anchor_lore(),
		ReaperGift::ReturnLight => words.gift_return_light_lore(),
		ReaperGift::Foreknowledge => words.gift_foreknowledge_lore(),
		ReaperGift::Verdict => words.gift_verdict_lore(),
		ReaperGift::Equilibrium => words.gift_equilibrium_lore(),
		ReaperGift::FirstLight => words.gift_first_light_lore(),
		ReaperGift::Reprisal => words.gift_reprisal_lore(),
		ReaperGift::LastWord => words.gift_last_word_lore(),
		ReaperGift::Hunger => words.gift_hunger_lore(),
		ReaperGift::Echo => words.gift_echo_lore(),
		ReaperGift::Unbroken => words.gift_unbroken_lore(),
		ReaperGift::Hush => words.gift_hush_lore(),
		ReaperGift::Keening => words.gift_keening_lore(),
		ReaperGift::Refrain => words.gift_refrain_lore(),
	}
}

/// What a gift does at `strength`. Every changing value is taken from the
/// same rules the simulation runs.
pub fn gift_effect(words: &dyn Messages, gift: ReaperGift, strength: i64) -> String {
	let bonus = (SoulRules::GIFT_STRIKE_PER_STRENGTH * strength as f64 * 100.0).round() as i64;
	let seconds = |ticks: i64| format!("{:.1}", ticks as f64 / HuntRules::TICK_RATE as f64);
	match gift {
		ReaperGift::Furrow => {
			words.gift_furrow_effect(
				((HuntRules::CLEAVE_SHARE
					+ SoulRules::FURROW_CLEAVE_PER_STRENGTH * strength as f64)
					* 100.0)
					.round() as i64,
			)
		}
		ReaperGift::Aftermath => {
			words.gift_aftermath_effect(&seconds(
				HuntRules::EXPOSED_TICKS + SoulRules::GATHERING_TICKS_PER_STRENGTH * strength,
			))
		}
		ReaperGift::Gathering => {
			words.gift_gathering_effect(&seconds(
				HuntRules::SEAL_COOLDOWN_TICKS + SoulRules::GATHERING_TICKS_PER_STRENGTH * strength,
			))
		}
		ReaperGift::Passage => {
			words.gift_passage_effect(&seconds(
				SoulRules::PASSAGE_STAGGER_TICKS_PER_STRENGTH * strength,
			))
		}
		ReaperGift::Anchor => words.gift_anchor_effect(strength),
		ReaperGift::ReturnLight => words.gift_return_light_effect(strength),
		ReaperGift::Foreknowledge => {
			words.gift_foreknowledge_effect(&seconds(
				SoulRules::FOREKNOWLEDGE_TICKS_PER_STRENGTH * strength,
			))
		}
		ReaperGift::Verdict => words.gift_verdict_effect(bonus),
		ReaperGift::Equilibrium => {
			words.gift_equilibrium_effect(&seconds(
				HuntRules::SEAL_COOLDOWN_TICKS
					+ SoulRules::EQUILIBRIUM_TICKS_PER_STRENGTH * strength,
			))
		}
		ReaperGift::FirstLight => words.gift_first_light_effect(bonus),
		ReaperGift::Reprisal => words.gift_reprisal_effect(bonus),
		ReaperGift::LastWord => words.gift_last_word_effect(bonus),
		ReaperGift::Hunger => words.gift_hunger_effect(strength),
		ReaperGift::Echo => words.gift_echo_effect(strength),
		ReaperGift::Unbroken => words.gift_unbroken_effect(strength),
		ReaperGift::Hush => {
			words.gift_hush_effect(&seconds(SoulRules::HUSH_TICKS_PER_STRENGTH * strength))
		}
		ReaperGift::Keening => words.gift_keening_effect(bonus),
		ReaperGift::Refrain => {
			words.gift_refrain_effect(&seconds(SoulRules::REFRAIN_TICKS_PER_STRENGTH * strength))
		}
	}
}

/// What the player is told when the cluster or their wallet did not do its
/// part. The program's own errors are named one by one, so a new one
/// cannot ship without words.
pub fn chain_failure_message(words: &dyn Messages, value: &ChainFailure) -> &'static str {
	match value {
		ChainFailure::NoWallet => words.chain_no_wallet(),
		ChainFailure::Declined => words.chain_declined(),
		ChainFailure::WrongWallet => words.chain_wrong_wallet(),
		ChainFailure::Rejected(_) => words.chain_rejected(),
		ChainFailure::Unreachable(_) => words.chain_unreachable(),
		ChainFailure::NotFinalized(_) => words.chain_not_finalized(),
		ChainFailure::Program(error) => program_error_message(words, error),
	}
}

pub fn program_error_message(words: &dyn Messages, value: &ReapersProgramError) -> &'static str {
	match value {
		ReapersProgramError::Paused => words.chain_paused(),
		ReapersProgramError::NotOwner => words.chain_not_owner(),
		ReapersProgramError::NotAnNft
		| ReapersProgramError::UnsupportedExtension
		| ReapersProgramError::SupplyNotFixed => words.chain_not_an_nft(),
		ReapersProgramError::MissingReceipt
		| ReapersProgramError::UnsupportedReceiptInstruction
		| ReapersProgramError::MalformedReceipt => words.chain_bad_receipt(),
		ReapersProgramError::WrongKeeper => words.chain_wrong_keeper(),
		ReapersProgramError::WrongPlayer => words.chain_wrong_player(),
		ReapersProgramError::WrongHusk => words.chain_wrong_husk(),
		ReapersProgramError::EscrowEmpty => words.chain_escrow_empty(),
		ReapersProgramError::NotAuthority
		| ReapersProgramError::NotUpgradeAuthority
		| ReapersProgramError::WrongKind => words.chain_program_refused(),
	}
}

pub fn cluster_name(words: &dyn Messages, value: ClusterName) -> &'static str {
	match value {
		ClusterName::Devnet => words.chain_devnet(),
		ClusterName::Localnet => words.chain_localnet(),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::l10n::En;

	#[test]
	fn every_refusal_has_words() {
		for refusal in Refusal::ALL {
			assert_ne!(refusal_message(&En, refusal), "", "{refusal:?}");
		}
	}

	#[test]
	fn every_gift_says_what_it_does_at_every_strength() {
		for gift in ReaperGift::ALL {
			for strength in 1..=4 {
				assert_ne!(gift_effect(&En, gift, strength), "");
			}
		}
	}

	#[test]
	fn every_class_has_a_name_and_a_line() {
		for reaper_class in ReaperClass::ALL {
			assert_ne!(class_name(&En, reaper_class), "");
			assert_ne!(class_line(&En, reaper_class), "");
		}
	}
}
