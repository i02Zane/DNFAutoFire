//! DNF 职业权威表：后端持有固定识别 ID，前端只通过命令读取展示模型。

use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct ClassInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub detection_index: u16,
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct ClassCategory {
    pub name: &'static str,
    pub classes: &'static [ClassInfo],
}

pub(crate) const CLASS_CATEGORIES: &[ClassCategory] = &[
    ClassCategory {
        name: "鬼剑士(男)",
        classes: &[
            ClassInfo {
                id: "male_slayer_blade_master",
                name: "剑魂",
                detection_index: 0,
            },
            ClassInfo {
                id: "male_slayer_soul_bender",
                name: "鬼泣",
                detection_index: 1,
            },
            ClassInfo {
                id: "male_slayer_berserker",
                name: "狂战士",
                detection_index: 2,
            },
            ClassInfo {
                id: "male_slayer_asura",
                name: "阿修罗",
                detection_index: 3,
            },
            ClassInfo {
                id: "male_slayer_ghostblade",
                name: "剑影",
                detection_index: 4,
            },
        ],
    },
    ClassCategory {
        name: "鬼剑士(女)",
        classes: &[
            ClassInfo {
                id: "female_slayer_sword_master",
                name: "驭剑士",
                detection_index: 5,
            },
            ClassInfo {
                id: "female_slayer_demon_slayer",
                name: "契魔者",
                detection_index: 6,
            },
            ClassInfo {
                id: "female_slayer_vagabond",
                name: "流浪武士",
                detection_index: 7,
            },
            ClassInfo {
                id: "female_slayer_dark_templar",
                name: "暗殿骑士",
                detection_index: 8,
            },
            ClassInfo {
                id: "female_slayer_spectre",
                name: "刃影",
                detection_index: 9,
            },
        ],
    },
    ClassCategory {
        name: "格斗家(男)",
        classes: &[
            ClassInfo {
                id: "male_fighter_nen_master",
                name: "气功师(男)",
                detection_index: 10,
            },
            ClassInfo {
                id: "male_fighter_striker",
                name: "散打(男)",
                detection_index: 11,
            },
            ClassInfo {
                id: "male_fighter_brawler",
                name: "街霸(男)",
                detection_index: 12,
            },
            ClassInfo {
                id: "male_fighter_grappler",
                name: "柔道家(男)",
                detection_index: 13,
            },
        ],
    },
    ClassCategory {
        name: "格斗家(女)",
        classes: &[
            ClassInfo {
                id: "female_fighter_nen_master",
                name: "气功师(女)",
                detection_index: 14,
            },
            ClassInfo {
                id: "female_fighter_striker",
                name: "散打(女)",
                detection_index: 15,
            },
            ClassInfo {
                id: "female_fighter_brawler",
                name: "街霸(女)",
                detection_index: 16,
            },
            ClassInfo {
                id: "female_fighter_grappler",
                name: "柔道家(女)",
                detection_index: 17,
            },
        ],
    },
    ClassCategory {
        name: "神枪手(男)",
        classes: &[
            ClassInfo {
                id: "male_gunner_ranger",
                name: "漫游枪手(男)",
                detection_index: 18,
            },
            ClassInfo {
                id: "male_gunner_launcher",
                name: "枪炮师(男)",
                detection_index: 19,
            },
            ClassInfo {
                id: "male_gunner_mechanic",
                name: "机械师(男)",
                detection_index: 20,
            },
            ClassInfo {
                id: "male_gunner_spitfire",
                name: "弹药专家(男)",
                detection_index: 21,
            },
            ClassInfo {
                id: "male_gunner_blitz",
                name: "合金战士",
                detection_index: 22,
            },
        ],
    },
    ClassCategory {
        name: "神枪手(女)",
        classes: &[
            ClassInfo {
                id: "female_gunner_ranger",
                name: "漫游枪手(女)",
                detection_index: 23,
            },
            ClassInfo {
                id: "female_gunner_launcher",
                name: "枪炮师(女)",
                detection_index: 24,
            },
            ClassInfo {
                id: "female_gunner_mechanic",
                name: "机械师(女)",
                detection_index: 25,
            },
            ClassInfo {
                id: "female_gunner_spitfire",
                name: "弹药专家(女)",
                detection_index: 26,
            },
            ClassInfo {
                id: "female_gunner_paramedic",
                name: "协战师",
                detection_index: 27,
            },
        ],
    },
    ClassCategory {
        name: "魔法师(男)",
        classes: &[
            ClassInfo {
                id: "male_mage_elemental_bomber",
                name: "元素爆破师",
                detection_index: 28,
            },
            ClassInfo {
                id: "male_mage_glacial_master",
                name: "冰结师",
                detection_index: 29,
            },
            ClassInfo {
                id: "male_mage_blood_mage",
                name: "猩红法师",
                detection_index: 30,
            },
            ClassInfo {
                id: "male_mage_swift_master",
                name: "逐风者",
                detection_index: 31,
            },
            ClassInfo {
                id: "male_mage_dimension_walker",
                name: "次元行者",
                detection_index: 32,
            },
        ],
    },
    ClassCategory {
        name: "魔法师(女)",
        classes: &[
            ClassInfo {
                id: "female_mage_elementalist",
                name: "元素师",
                detection_index: 33,
            },
            ClassInfo {
                id: "female_mage_summoner",
                name: "召唤师",
                detection_index: 34,
            },
            ClassInfo {
                id: "female_mage_battle_mage",
                name: "战斗法师",
                detection_index: 35,
            },
            ClassInfo {
                id: "female_mage_witch",
                name: "魔道学者",
                detection_index: 36,
            },
            ClassInfo {
                id: "female_mage_enchantress",
                name: "小魔女",
                detection_index: 37,
            },
        ],
    },
    ClassCategory {
        name: "光职者(男)",
        classes: &[
            ClassInfo {
                id: "male_priest_crusader",
                name: "光明骑士(男)",
                detection_index: 38,
            },
            ClassInfo {
                id: "male_priest_monk",
                name: "蓝拳使者",
                detection_index: 39,
            },
            ClassInfo {
                id: "male_priest_exorcist",
                name: "驱魔师(男)",
                detection_index: 40,
            },
            ClassInfo {
                id: "male_priest_avenger",
                name: "惩戒者",
                detection_index: 41,
            },
        ],
    },
    ClassCategory {
        name: "光职者(女)",
        classes: &[
            ClassInfo {
                id: "female_priest_crusader",
                name: "光明骑士(女)",
                detection_index: 42,
            },
            ClassInfo {
                id: "female_priest_inquisitor",
                name: "正义审判者",
                detection_index: 43,
            },
            ClassInfo {
                id: "female_priest_shaman",
                name: "驱魔师(女)",
                detection_index: 44,
            },
            ClassInfo {
                id: "female_priest_mistress",
                name: "除恶者",
                detection_index: 45,
            },
        ],
    },
    ClassCategory {
        name: "暗夜使者",
        classes: &[
            ClassInfo {
                id: "female_thief_rogue",
                name: "暗星",
                detection_index: 46,
            },
            ClassInfo {
                id: "female_thief_necromancer",
                name: "黑夜术士",
                detection_index: 47,
            },
            ClassInfo {
                id: "female_thief_kunoichi",
                name: "忍者",
                detection_index: 48,
            },
            ClassInfo {
                id: "female_thief_shadow_dancer",
                name: "影舞者",
                detection_index: 49,
            },
        ],
    },
    ClassCategory {
        name: "守护者",
        classes: &[
            ClassInfo {
                id: "female_knight_elven_knight",
                name: "精灵骑士",
                detection_index: 50,
            },
            ClassInfo {
                id: "female_knight_chaos",
                name: "混沌魔灵",
                detection_index: 51,
            },
            ClassInfo {
                id: "female_knight_lightbringer",
                name: "帕拉丁",
                detection_index: 52,
            },
            ClassInfo {
                id: "female_knight_dragon_knight",
                name: "龙骑士",
                detection_index: 53,
            },
        ],
    },
    ClassCategory {
        name: "魔枪士",
        classes: &[
            ClassInfo {
                id: "male_demonic_lancer_vanguard",
                name: "征战者",
                detection_index: 54,
            },
            ClassInfo {
                id: "male_demonic_lancer_skirmisher",
                name: "决战者",
                detection_index: 55,
            },
            ClassInfo {
                id: "male_demonic_lancer_dragoon",
                name: "狩猎者",
                detection_index: 56,
            },
            ClassInfo {
                id: "male_demonic_lancer_impaler",
                name: "暗枪士",
                detection_index: 57,
            },
        ],
    },
    ClassCategory {
        name: "枪剑士",
        classes: &[
            ClassInfo {
                id: "male_agent_secret_agent",
                name: "暗刃",
                detection_index: 58,
            },
            ClassInfo {
                id: "male_agent_troubleshooter",
                name: "特工",
                detection_index: 59,
            },
            ClassInfo {
                id: "male_agent_hitman",
                name: "战线佣兵",
                detection_index: 60,
            },
            ClassInfo {
                id: "male_agent_specialist",
                name: "源能专家",
                detection_index: 61,
            },
        ],
    },
    ClassCategory {
        name: "弓箭手",
        classes: &[
            ClassInfo {
                id: "female_archer_muse",
                name: "缪斯",
                detection_index: 62,
            },
            ClassInfo {
                id: "female_archer_traveler",
                name: "旅人",
                detection_index: 63,
            },
            ClassInfo {
                id: "female_archer_hunter",
                name: "猎人",
                detection_index: 64,
            },
            ClassInfo {
                id: "female_archer_vigilante",
                name: "妖护使",
                detection_index: 65,
            },
            ClassInfo {
                id: "female_archer_chimera",
                name: "奇美拉",
                detection_index: 66,
            },
        ],
    },
    ClassCategory {
        name: "外传",
        classes: &[
            ClassInfo {
                id: "male_dark_knight",
                name: "黑暗武士",
                detection_index: 67,
            },
            ClassInfo {
                id: "female_creator",
                name: "缔造者",
                detection_index: 68,
            },
        ],
    },
];

pub(crate) fn class_categories() -> Vec<ClassCategory> {
    CLASS_CATEGORIES.to_vec()
}

pub(crate) fn class_id_by_detection_index(detection_index: u16) -> Option<&'static str> {
    class_infos()
        .find(|class_info| class_info.detection_index == detection_index)
        .map(|class_info| class_info.id)
}

pub(crate) fn class_name_by_id(class_id: &str) -> Option<&'static str> {
    class_infos()
        .find(|class_info| class_info.id == class_id)
        .map(|class_info| class_info.name)
}

fn class_infos() -> impl Iterator<Item = &'static ClassInfo> {
    CLASS_CATEGORIES
        .iter()
        .flat_map(|category| category.classes.iter())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detection_indexes_are_unique() {
        let mut indexes = std::collections::HashSet::new();
        for class_info in class_infos() {
            assert!(
                indexes.insert(class_info.detection_index),
                "duplicate detection index {}",
                class_info.detection_index
            );
        }
    }

    #[test]
    fn detection_index_maps_to_stable_class_id() {
        assert_eq!(
            class_id_by_detection_index(0),
            Some("male_slayer_blade_master")
        );
        assert_eq!(
            class_id_by_detection_index(63),
            Some("female_archer_traveler")
        );
    }

    #[test]
    fn class_id_maps_to_display_name() {
        assert_eq!(class_name_by_id("male_slayer_blade_master"), Some("剑魂"));
        assert_eq!(class_name_by_id("missing"), None);
    }
}
