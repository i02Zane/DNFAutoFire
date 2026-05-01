// DNF 职业静态索引：职业 id 持久化到配置文件，name 只用于界面展示。
export type ClassInfo = {
  id: string;
  name: string;
};

export type ClassCategory = {
  name: string;
  classes: ClassInfo[];
};

export const classCategories: ClassCategory[] = [
  {
    name: "鬼剑士(男)",
    classes: [
      { id: "male_slayer_blade_master", name: "剑魂" },
      { id: "male_slayer_soul_bender", name: "鬼泣" },
      { id: "male_slayer_berserker", name: "狂战士" },
      { id: "male_slayer_asura", name: "阿修罗" },
      { id: "male_slayer_ghostblade", name: "剑影" },
    ],
  },
  {
    name: "鬼剑士(女)",
    classes: [
      { id: "female_slayer_sword_master", name: "驭剑士" },
      { id: "female_slayer_demon_slayer", name: "契魔者" },
      { id: "female_slayer_vagabond", name: "流浪武士" },
      { id: "female_slayer_dark_templar", name: "暗殿骑士" },
      { id: "female_slayer_spectre", name: "刃影" },
    ],
  },
  {
    name: "格斗家(男)",
    classes: [
      { id: "male_fighter_nen_master", name: "气功师(男)" },
      { id: "male_fighter_striker", name: "散打(男)" },
      { id: "male_fighter_brawler", name: "街霸(男)" },
      { id: "male_fighter_grappler", name: "柔道家(男)" },
    ],
  },
  {
    name: "格斗家(女)",
    classes: [
      { id: "female_fighter_nen_master", name: "气功师(女)" },
      { id: "female_fighter_striker", name: "散打(女)" },
      { id: "female_fighter_brawler", name: "街霸(女)" },
      { id: "female_fighter_grappler", name: "柔道家(女)" },
    ],
  },
  {
    name: "神枪手(男)",
    classes: [
      { id: "male_gunner_ranger", name: "漫游枪手(男)" },
      { id: "male_gunner_launcher", name: "枪炮师(男)" },
      { id: "male_gunner_mechanic", name: "机械师(男)" },
      { id: "male_gunner_spitfire", name: "弹药专家(男)" },
      { id: "male_gunner_blitz", name: "合金战士" },
    ],
  },
  {
    name: "神枪手(女)",
    classes: [
      { id: "female_gunner_ranger", name: "漫游枪手(女)" },
      { id: "female_gunner_launcher", name: "枪炮师(女)" },
      { id: "female_gunner_mechanic", name: "机械师(女)" },
      { id: "female_gunner_spitfire", name: "弹药专家(女)" },
      { id: "female_gunner_paramedic", name: "协战师" },
    ],
  },
  {
    name: "魔法师(男)",
    classes: [
      { id: "male_mage_elemental_bomber", name: "元素爆破师" },
      { id: "male_mage_glacial_master", name: "冰结师" },
      { id: "male_mage_blood_mage", name: "猩红法师" },
      { id: "male_mage_swift_master", name: "逐风者" },
      { id: "male_mage_dimension_walker", name: "次元行者" },
    ],
  },
  {
    name: "魔法师(女)",
    classes: [
      { id: "female_mage_elementalist", name: "元素师" },
      { id: "female_mage_summoner", name: "召唤师" },
      { id: "female_mage_battle_mage", name: "战斗法师" },
      { id: "female_mage_witch", name: "魔道学者" },
      { id: "female_mage_enchantress", name: "小魔女" },
    ],
  },
  {
    name: "光职者(男)",
    classes: [
      { id: "male_priest_crusader", name: "光明骑士(男)" },
      { id: "male_priest_monk", name: "蓝拳使者" },
      { id: "male_priest_exorcist", name: "驱魔师(男)" },
      { id: "male_priest_avenger", name: "惩戒者" },
    ],
  },
  {
    name: "光职者(女)",
    classes: [
      { id: "female_priest_crusader", name: "光明骑士(女)" },
      { id: "female_priest_inquisitor", name: "正义审判者" },
      { id: "female_priest_shaman", name: "驱魔师(女)" },
      { id: "female_priest_mistress", name: "除恶者" },
    ],
  },
  {
    name: "暗夜使者",
    classes: [
      { id: "female_thief_rogue", name: "暗星" },
      { id: "female_thief_necromancer", name: "黑夜术士" },
      { id: "female_thief_kunoichi", name: "忍者" },
      { id: "female_thief_shadow_dancer", name: "影舞者" },
    ],
  },
  {
    name: "守护者",
    classes: [
      { id: "female_knight_elven_knight", name: "精灵骑士" },
      { id: "female_knight_chaos", name: "混沌魔灵" },
      { id: "female_knight_lightbringer", name: "帕拉丁" },
      { id: "female_knight_dragon_knight", name: "龙骑士" },
    ],
  },
  {
    name: "魔枪士",
    classes: [
      { id: "male_demonic_lancer_vanguard", name: "征战者" },
      { id: "male_demonic_lancer_skirmisher", name: "决战者" },
      { id: "male_demonic_lancer_dragoon", name: "狩猎者" },
      { id: "male_demonic_lancer_impaler", name: "暗枪士" },
    ],
  },
  {
    name: "枪剑士",
    classes: [
      { id: "male_agent_secret_agent", name: "暗刃" },
      { id: "male_agent_troubleshooter", name: "特工" },
      { id: "male_agent_hitman", name: "战线佣兵" },
      { id: "male_agent_specialist", name: "源能专家" },
    ],
  },
  {
    name: "弓箭手",
    classes: [
      { id: "female_archer_muse", name: "缪斯" },
      { id: "female_archer_traveler", name: "旅人" },
      { id: "female_archer_hunter", name: "猎人" },
      { id: "female_archer_vigilante", name: "妖护使" },
      { id: "female_archer_chimera", name: "奇美拉" },
    ],
  },
  {
    name: "外传",
    classes: [
      { id: "male_dark_knight", name: "黑暗武士" },
      { id: "female_creator", name: "缔造者" },
    ],
  },
];

export function getClassName(classId: string): string {
  return getClassInfo(classId)?.name ?? "未知职业";
}

export function getClassInfo(classId: string): ClassInfo | undefined {
  return classCategories
    .flatMap((category) => category.classes)
    .find((classInfo) => classInfo.id === classId);
}
