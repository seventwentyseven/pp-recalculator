use bitflags::bitflags;

// Create a bitflags struct
bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Mods: u32 {
        const NoMod = 0;
        const NoFail = 1 << 0;
        const Easy = 1 << 1;
        const TouchScreen = 1 << 2;  // old: 'NOVIDEO'
        const Hidden = 1 << 3;
        const HardRock = 1 << 4;
        const SuddenDeath = 1 << 5;
        const DoubleTime = 1 << 6;
        const Relax = 1 << 7;
        const HalfTime = 1 << 8;
        const NightCore = 1 << 9;
        const FlashLight = 1 << 10;
        const AutoPlay = 1 << 11;
        const SpunOut = 1 << 12;
        const AutoPilot = 1 << 13;
        const Perfect = 1 << 14;
        const Key4 = 1 << 15;
        const Key5 = 1 << 16;
        const Key6 = 1 << 17;
        const Key7 = 1 << 18;
        const Key8 = 1 << 19;
        const FadeIn = 1 << 20;
        const Random = 1 << 21;
        const Cinema = 1 << 22;
        const Target = 1 << 23;
        const Key9 = 1 << 24;
        const KeyCOOP = 1 << 25;
        const Key1 = 1 << 26;
        const Key3 = 1 << 27;
        const Key2 = 1 << 28;
        const ScoreV2 = 1 << 29;
        const Mirror = 1 << 30;
    }
}
