///!

//! # Protocolo TLV Binário v2.0.0 — Tipos e Definições
//!
//! Este módulo define todas as constantes, enums e structs utilizados pelo protocolo
//! de comunicação TLV (Type-Length-Value) do sistema AERUS VISOR.
//!
//! O protocolo é partilhado por todos os módulos e sistemas do AERUS, sendo
//! compatível com retroalimentação (backward-compatible) através de regras semânticas
//! de versão: apenas adicionar novos campos/IDs, nunca alterar os existentes.
//!
//! ## Formato da Mensagem
//!
//! ```text
//! [START_BYTE][MSGID][TLV_COUNT][TLV_FIELDS...][CRC8]
//! ```
//!
## Endianness: Todos os campos multi-byte utilizam little-endian (LE).

// ============================================================================
// CONSTANTES GLOBAIS
// ============================================================================

/// Byte de início de cada mensagem TLV. Usado para sincronização de frame.
pub const START_BYTE: u8 = 0xAA;

/// Número máximo de bytes de dados em um campo TLV normal.
pub const MAX_TLV_DATA: usize = 32;

/// Número máximo de bytes de dados em um campo TLV de vídeo.
pub const MAX_TLV_VIDEO_DATA: usize = 128;

/// Número máximo de campos TLV por mensagem.
pub const MAX_TLV_FIELDS: usize = 32;

/// Tamanho máximo de uma mensagem TLV serializada em bytes.
/// Calculado como: 1 (start) + 1 (msgID) + 1 (tlvCount) + 32 * (1+1+32) + 1 (crc) = 1093
pub const MAX_MESSAGE_SIZE: usize = 1093;

/// Tamanho do cabeçalho da mensagem TLV (start + msgID + tlvCount).
pub const MESSAGE_HEADER_SIZE: usize = 3;

/// Tamanho do checksum CRC8 no final da mensagem.
pub const CHECKSUM_SIZE: usize = 1;

/// Tamanho do cabeçalho de cada campo TLV (id + len).
pub const TLV_HEADER_SIZE: usize = 2;

// ============================================================================
// ENUMS — IDENTIFICADORES DE MENSAGEM
// ============================================================================

/// Identificadores de mensagem (MsgID).
///
/// Cada tipo de mensagem tem um ID único que define o seu propósito
/// e o conjunto de campos TLV que contém.
///
/// Valores na faixa 0x10-0x1F para mensagens do sistema.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MsgId {
    /// Heartbeat — sinal de vida do módulo, enviado periodicamente.
    Heartbeat = 0x10,
    /// Telemetria — dados de sensores e estado do sistema.
    Telemetry = 0x11,
    /// Comando — instrução enviada ao módulo.
    Command = 0x12,
    /// Confirmação (ACK) — confirmação de receção de mensagem.
    Ack = 0x13,
    /// Failsafe — estado de segurança de emergência.
    Failsafe = 0x14,
    /// Debug — mensagens de depuração.
    Debug = 0x15,
    /// Vídeo — dados de vídeo fragmentados.
    Video = 0x16,
    /// Shell — acesso a consola remota.
    Shell = 0x17,
    /// Dados de sensores SI (Sensor Interface).
    SiData = 0x18,
}

impl MsgId {
    /// Converte um valor u8 para MsgId, retornando None se inválido.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x10 => Some(Self::Heartbeat),
            0x11 => Some(Self::Telemetry),
            0x12 => Some(Self::Command),
            0x13 => Some(Self::Ack),
            0x14 => Some(Self::Failsafe),
            0x15 => Some(Self::Debug),
            0x16 => Some(Self::Video),
            0x17 => Some(Self::Shell),
            0x18 => Some(Self::SiData),
            _ => None,
        }
    }

    /// Retorna true se o ID da mensagem é válido.
    pub fn is_valid(id: u8) -> bool {
        Self::from_u8(id).is_some()
    }
}

// ============================================================================
// ENUMS — NÍVEIS DE PRIORIDADE
// ============================================================================

/// Níveis de prioridade para ordenação de mensagens.
///
/// Valores menores indicam prioridade mais alta.
/// O sistema utiliza esta escala para garantir que mensagens críticas
/// (como failsafe) são processadas antes de mensagens de vídeo ou debug.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PriorityLevel {
    /// Crítico — processamento imediato, não pode ser adiado.
    SuperCritical = 0,
    /// Alta — processamento urgente.
    High = 1,
    /// Normal — processamento padrão.
    Normal = 2,
    /// Baixa — processamento quando disponível.
    Low = 3,
    /// Muito baixa — apenas quando ocioso.
    SuperLow = 4,
}

impl PriorityLevel {
    /// Converte um valor u8 para PriorityLevel.
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::SuperCritical,
            1 => Self::High,
            2 => Self::Normal,
            3 => Self::Low,
            _ => Self::SuperLow,
        }
    }
}

// ============================================================================
// ENUMS — ESTADO DO SISTEMA
// ============================================================================

/// Estados possíveis do sistema.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemState {
    Booting = 0,
    Initializing = 1,
    Ready = 2,
    Armed = 3,
    InFlight = 4,
    Landing = 5,
    Error = 6,
    Shutdown = 7,
}

impl SystemState {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Booting),
            1 => Some(Self::Initializing),
            2 => Some(Self::Ready),
            3 => Some(Self::Armed),
            4 => Some(Self::InFlight),
            5 => Some(Self::Landing),
            6 => Some(Self::Error),
            7 => Some(Self::Shutdown),
            _ => None,
        }
    }
}

// ============================================================================
// ENUMS — MODO DE VOO
// ============================================================================

/// Modos de voo disponíveis.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlightMode {
    Manual = 0,
    Stabilize = 1,
    AltHold = 2,
    Auto = 3,
    Guided = 4,
    Rtl = 5,
}

impl FlightMode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Manual),
            1 => Some(Self::Stabilize),
            2 => Some(Self::AltHold),
            3 => Some(Self::Auto),
            4 => Some(Self::Guided),
            5 => Some(Self::Rtl),
            _ => None,
        }
    }
}

// ============================================================================
// ENUMS — MOTIVO E AÇÃO DE FAILSAFE
// ============================================================================

/// Motivos para ativação do failsafe.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailsafeReason {
    None = 0,
    SignalLost = 1,
    LowBattery = 2,
    GpsLost = 3,
    SensorFailure = 4,
    ManualTrigger = 5,
}

impl FailsafeReason {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::SignalLost),
            2 => Some(Self::LowBattery),
            3 => Some(Self::GpsLost),
            4 => Some(Self::SensorFailure),
            5 => Some(Self::ManualTrigger),
            _ => None,
        }
    }
}

/// Ações a tomar quando o failsafe é ativado.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailsafeAction {
    None = 0,
    Hover = 1,
    Land = 2,
    Rtl = 3,
    Continue = 4,
    Disarm = 5,
}

impl FailsafeAction {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Hover),
            2 => Some(Self::Land),
            3 => Some(Self::Rtl),
            4 => Some(Self::Continue),
            5 => Some(Self::Disarm),
            _ => None,
        }
    }
}

// ============================================================================
// ENUMS — CAMPOS TLV (GPS, IMU, Voo, Energia, Temperatura, Sistema, etc.)
// ============================================================================

/// Identificadores de campos TLV para dados GPS.
///
/// Faixa de IDs: 0x20-0x2F.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldGps {
    Latitude = 0x20,
    Longitude = 0x21,
    Altitude = 0x22,
    Speed = 0x23,
    Course = 0x24,
    Satellites = 0x25,
    Hdop = 0x26,
}

/// Identificadores de campos TLV para dados IMU (Unidade de Medição Inercial).
///
/// Faixa de IDs: 0x30-0x3F.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldImu {
    Roll = 0x30,
    Pitch = 0x31,
    Yaw = 0x32,
    AccelX = 0x33,
    AccelY = 0x34,
    AccelZ = 0x35,
    GyroX = 0x36,
    GyroY = 0x37,
    GyroZ = 0x38,
    YawRate = 0x39,
}

/// Identificadores de campos TLV para dados de voo.
///
/// Faixa de IDs: 0x40-0x4F.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldFlight {
    AltGps = 0x40,
    AltBaro = 0x41,
    VSpeed = 0x42,
    Airspeed = 0x43,
    LoopTime = 0x44,
}

/// Identificadores de campos TLV para dados de energia.
///
/// Faixa de IDs: 0x50-0x5F.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldPower {
    BattVoltage = 0x50,
    BattCurrent = 0x51,
    BattConsumed = 0x52,
    BattTemp = 0x53,
    BattSoc = 0x54,
}

/// Identificadores de campos TLV para dados de temperatura.
///
/// Faixa de IDs: 0x60-0x6F.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldTemp {
    Temp1 = 0x60,
    Temp2 = 0x61,
    Temp3 = 0x62,
    Temp4 = 0x63,
    Esp1Temp = 0x64,
    Esp2Temp = 0x65,
}

/// Identificadores de campos TLV para dados de sistema.
///
/// Faixa de IDs: 0x70-0x7F.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldSystem {
    State = 0x70,
    Mode = 0x71,
    Uptime = 0x72,
    FreeHeap = 0x73,
    CpuLoad = 0x74,
    Esp1Load = 0x75,
    Esp2Load = 0x76,
}

/// Identificadores de campos TLV para dados de failsafe.
///
/// Faixa de IDs: 0xA1-0xAF.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldFailsafe {
    Reason = 0xA1,
    Action = 0xA2,
    State = 0xA3,
}

/// Identificadores de campos TLV para dados de vídeo.
///
/// Faixa de IDs: 0xB0-0xBF.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldVideo {
    FrameId = 0xB0,
    ChunkId = 0xB1,
    TotalChunks = 0xB2,
    Payload = 0xB3,
}

/// Unificação de todos os identificadores de campos TLV.
///
/// Este enum permite trabalhar com qualquer campo TLV de forma genérica,
/// enquanto mantém a tipagem para campos específicos.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldId {
    // GPS (0x20-0x2F)
    GpsLatitude = 0x20,
    GpsLongitude = 0x21,
    GpsAltitude = 0x22,
    GpsSpeed = 0x23,
    GpsCourse = 0x24,
    GpsSatellites = 0x25,
    GpsHdop = 0x26,

    // IMU (0x30-0x3F)
    ImuRoll = 0x30,
    ImuPitch = 0x31,
    ImuYaw = 0x32,
    ImuAccelX = 0x33,
    ImuAccelY = 0x34,
    ImuAccelZ = 0x35,
    ImuGyroX = 0x36,
    ImuGyroY = 0x37,
    ImuGyroZ = 0x38,
    ImuYawRate = 0x39,

    // Voo (0x40-0x4F)
    FlightAltGps = 0x40,
    FlightAltBaro = 0x41,
    FlightVSpeed = 0x42,
    FlightAirspeed = 0x43,
    FlightLoopTime = 0x44,

    // Energia (0x50-0x5F)
    PowerBattV = 0x50,
    PowerBattI = 0x51,
    PowerBattCons = 0x52,
    PowerBattTemp = 0x53,
    PowerBattSoc = 0x54,

    // Temperatura (0x60-0x6F)
    Temp1 = 0x60,
    Temp2 = 0x61,
    Temp3 = 0x62,
    Temp4 = 0x63,
    TempEsp1 = 0x64,
    TempEsp2 = 0x65,

    // Sistema (0x70-0x7F)
    SystemState = 0x70,
    SystemMode = 0x71,
    SystemUptime = 0x72,
    SystemFreeHeap = 0x73,
    SystemCpuLoad = 0x74,
    SystemEsp1Load = 0x75,
    SystemEsp2Load = 0x76,

    // Failsafe (0xA1-0xAF)
    FailsafeReason = 0xA1,
    FailsafeAction = 0xA2,
    FailsafeState = 0xA3,

    // Vídeo (0xB0-0xBF)
    VideoFrameId = 0xB0,
    VideoChunkId = 0xB1,
    VideoTotalChunks = 0xB2,
    VideoPayload = 0xB3,
}

impl FieldId {
    /// Converte um valor u8 para FieldId, retornando None se inválido.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x20..=0x2F => Some(unsafe { core::mem::transmute(value) }),
            0x30..=0x3F => Some(unsafe { core::mem::transmute(value) }),
            0x40..=0x4F => Some(unsafe { core::mem::transmute(value) }),
            0x50..=0x5F => Some(unsafe { core::mem::transmute(value) }),
            0x60..=0x6F => Some(unsafe { core::mem::transmute(value) }),
            0x70..=0x7F => Some(unsafe { core::mem::transmute(value) }),
            0xA1..=0xAF => Some(unsafe { core::mem::transmute(value) }),
            0xB0..=0xBF => Some(unsafe { core::mem::transmute(value) }),
            _ => None,
        }
    }

    /// Retorna true se o ID do campo é válido.
    pub fn is_valid(id: u8) -> bool {
        matches!(id,
            0x20..=0x2F | 0x30..=0x3F | 0x40..=0x4F | 0x50..=0x5F |
            0x60..=0x6F | 0x70..=0x7F | 0xA1..=0xAF | 0xB0..=0xBF
        )
    }
}

// ============================================================================
// ENUMS — COMANDOS
// ============================================================================

/// Comandos básicos do sistema.
///
/// Faixa de IDs: 0xC0-0xCF.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandBasic {
    Arm = 0xC0,
    Disarm = 0xC1,
    SetMode = 0xC2,
    EmergencyStop = 0xC3,
    Shutdown = 0xC4,
}

/// Comandos de controlo de voo.
///
/// Faixa de IDs: 0xD0-0xDF.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandControl {
    SetAltTarget = 0xD0,
    SetSpeed = 0xD1,
    SetPitch = 0xD2,
    SetRoll = 0xD3,
    SetYaw = 0xD4,
    SetHeading = 0xD5,
}

/// Comandos avançados de diagnóstico.
///
/// Faixa de IDs: 0xE0-0xEF.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandAdvanced {
    SensorCalib = 0xE0,
    StartLog = 0xE1,
    StopLog = 0xE2,
    GetAll = 0xE3,
}

/// Comandos de navegação.
///
/// Faixa de IDs: 0xF0-0xFF.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandNav {
    NextWaypoints = 0xF0,
    SetReturnPoint = 0xF1,
    SetPosition = 0xF2,
}

/// Unificação de todos os comandos.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Command {
    // Básicos (0xC0-0xCF)
    Arm = 0xC0,
    Disarm = 0xC1,
    SetMode = 0xC2,
    EmergencyStop = 0xC3,
    Shutdown = 0xC4,

    // Controlo (0xD0-0xDF)
    SetAltTarget = 0xD0,
    SetSpeed = 0xD1,
    SetPitch = 0xD2,
    SetRoll = 0xD3,
    SetYaw = 0xD4,
    SetHeading = 0xD5,

    // Avançados (0xE0-0xEF)
    SensorCalib = 0xE0,
    StartLog = 0xE1,
    StopLog = 0xE2,
    GetAll = 0xE3,

    // Navegação (0xF0-0xFF)
    NextWaypoints = 0xF0,
    SetReturnPoint = 0xF1,
    SetPosition = 0xF2,
}

// ============================================================================
// STRUCTS — CAMPOS E MENSAGENS TLV
// ============================================================================

/// Campo TLV (Type-Length-Value) genérico.
///
/// Representa um campo individual dentro de uma mensagem TLV.
/// O array `data` contém até `MAX_TLV_DATA` bytes de dados.
///
/// **Importante:** Este struct utiliza `#[repr(C)]` para garantir layout
/// compatível com C/C++ através de FFI.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TLVField {
    /// Identificador do tipo do campo.
    pub id: u8,
    /// Número de bytes de dados válidos no array `data`.
    pub len: u8,
    /// Dados do campo. Apenas os primeiros `len` bytes são válidos.
    pub data: [u8; MAX_TLV_DATA],
}

impl TLVField {
    /// Cria um novo campo TLV vazio.
    pub fn new() -> Self {
        Self {
            id: 0,
            len: 0,
            data: [0u8; MAX_TLV_DATA],
        }
    }

    /// Cria um novo campo TLV com o ID e dados especificados.
    pub fn with_data(id: u8, data: &[u8]) -> Self {
        let mut field = Self::new();
        field.id = id;
        let len = data.len().min(MAX_TLV_DATA);
        field.len = len as u8;
        field.data[..len].copy_from_slice(&data[..len]);
        field
    }
}

impl Default for TLVField {
    fn default() -> Self {
        Self::new()
    }
}

/// Campo TLV específico para dados de vídeo.
///
/// Utiliza `MAX_TLV_VIDEO_DATA` (128 bytes) em vez de `MAX_TLV_DATA` (32 bytes)
/// para acomodar payloads de vídeo maiores.
///
/// **Importante:** Este struct utiliza `#[repr(C)]` para garantir layout
/// compatível com C/C++ através de FFI.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TLVVideoField {
    /// Identificador do tipo do campo.
    pub id: u8,
    /// Número de bytes de dados válidos no array `data`.
    pub len: u8,
    /// Dados do campo de vídeo.
    pub data: [u8; MAX_TLV_VIDEO_DATA],
}

impl TLVVideoField {
    /// Cria um novo campo TLV de vídeo vazio.
    pub fn new() -> Self {
        Self {
            id: 0,
            len: 0,
            data: [0u8; MAX_TLV_VIDEO_DATA],
        }
    }
}

impl Default for TLVVideoField {
    fn default() -> Self {
        Self::new()
    }
}

/// Mensagem TLV completa.
///
/// Estrutura principal do protocolo. Contém o cabeçalho, até `MAX_TLV_FIELDS`
/// campos TLV e um checksum CRC8 para integridade.
///
/// Formato na memória:
/// ```text
/// [startByte][msgID][tlvCount][tlv[0]...tlv[N]][checksum]
/// ```
///
/// **Importante:** Este struct utiliza `#[repr(C)]` para garantir layout
/// compatível com C/C++ através de FFI. O tamanho total é fixo (1093 bytes).
#[repr(C)]
#[derive(Debug, Clone)]
pub struct TLVMessage {
    /// Byte de início (sempre `START_BYTE` = 0xAA).
    pub start_byte: u8,
    /// Identificador do tipo de mensagem.
    pub msg_id: u8,
    /// Número de campos TLV válidos na mensagem.
    pub tlv_count: u8,
    /// Array de campos TLV. Apenas os primeiros `tlv_count` são válidos.
    pub tlvs: [TLVField; MAX_TLV_FIELDS],
    /// Checksum CRC8 de toda a mensagem (start + msgID + tlvCount + tlvs).
    pub checksum: u8,
}

impl TLVMessage {
    /// Cria uma nova mensagem TLV vazia.
    pub fn new() -> Self {
        Self {
            start_byte: START_BYTE,
            msg_id: 0,
            tlv_count: 0,
            tlvs: [TLVField::new(); MAX_TLV_FIELDS],
            checksum: 0,
        }
    }

    /// Cria uma nova mensagem TLV com o ID de mensagem especificado.
    pub fn with_id(msg_id: u8) -> Self {
        Self {
            start_byte: START_BYTE,
            msg_id,
            tlv_count: 0,
            tlvs: [TLVField::new(); MAX_TLV_FIELDS],
            checksum: 0,
        }
    }

    /// Limpa a mensagem, redefinindo todos os campos para os valores padrão.
    pub fn clear(&mut self) {
        self.start_byte = START_BYTE;
        self.msg_id = 0;
        self.tlv_count = 0;
        for tlv in self.tlvs.iter_mut() {
            *tlv = TLVField::new();
        }
        self.checksum = 0;
    }
}

impl Default for TLVMessage {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// FUNÇÕES DE CONVERSÃO DE BYTES (Little-Endian)
// ============================================================================

/// Converte um valor f32 para 4 bytes em formato little-endian.
pub fn float_to_bytes(value: f32) -> [u8; 4] {
    value.to_le_bytes()
}

/// Converte 4 bytes em formato little-endian para f32.
pub fn bytes_to_float(bytes: &[u8; 4]) -> f32 {
    f32::from_le_bytes(*bytes)
}

/// Converte um valor i32 para 4 bytes em formato little-endian.
pub fn int32_to_bytes(value: i32) -> [u8; 4] {
    value.to_le_bytes()
}

/// Converte 4 bytes em formato little-endian para i32.
pub fn bytes_to_int32(bytes: &[u8; 4]) -> i32 {
    i32::from_le_bytes(*bytes)
}

/// Converte um valor u32 para 4 bytes em formato little-endian.
pub fn uint32_to_bytes(value: u32) -> [u8; 4] {
    value.to_le_bytes()
}

/// Converte 4 bytes em formato little-endian para u32.
pub fn bytes_to_uint32(bytes: &[u8; 4]) -> u32 {
    u32::from_le_bytes(*bytes)
}

/// Converte um valor u16 para 2 bytes em formato little-endian.
pub fn uint16_to_bytes(value: u16) -> [u8; 2] {
    value.to_le_bytes()
}

/// Converte 2 bytes em formato little-endian para u16.
pub fn bytes_to_uint16(bytes: &[u8; 2]) -> u16 {
    u16::from_le_bytes(*bytes)
}

// ============================================================================
// FUNÇÕES DE VALIDAÇÃO
// ============================================================================

/// Retorna true se o ID da mensagem é válido.
pub fn is_valid_msg_id(id: u8) -> bool {
    MsgId::is_valid(id)
}

/// Retorna true se o ID do campo TLV é válido.
pub fn is_valid_field_id(id: u8) -> bool {
    FieldId::is_valid(id)
}

/// Retorna a prioridade de uma mensagem com base no seu ID e estado de failsafe.
///
/// Se o failsafe estiver ativo, todas as mensagens recebem prioridade SUPER_CRITICAL,
/// exceto mensagens de debug que ficam em SUPER_LOW.
pub fn get_msg_priority(msg_id: u8, failsafe_active: bool) -> u8 {
    if failsafe_active {
        return if msg_id == MsgId::Debug as u8 {
            PriorityLevel::SuperLow as u8
        } else {
            PriorityLevel::SuperCritical as u8
        };
    }

    match msg_id {
        0x10 => PriorityLevel::Normal as u8,     // Heartbeat
        0x11 => PriorityLevel::Normal as u8,     // Telemetry
        0x12 => PriorityLevel::High as u8,       // Command
        0x13 => PriorityLevel::High as u8,       // ACK
        0x14 => PriorityLevel::SuperCritical as u8, // Failsafe
        0x15 => PriorityLevel::SuperLow as u8,   // Debug
        0x16 => PriorityLevel::Low as u8,        // Video
        0x17 => PriorityLevel::Normal as u8,     // Shell
        0x18 => PriorityLevel::Normal as u8,     // SI Data
        _ => PriorityLevel::SuperLow as u8,
    }
}

// ============================================================================
// TESTES UNITÁRIOS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msg_id_validity() {
        assert!(MsgId::is_valid(0x10));
        assert!(MsgId::is_valid(0x18));
        assert!(!MsgId::is_valid(0x00));
        assert!(!MsgId::is_valid(0x19));
    }

    #[test]
    fn test_field_id_validity() {
        assert!(FieldId::is_valid(0x20));
        assert!(FieldId::is_valid(0x30));
        assert!(FieldId::is_valid(0xB3));
        assert!(!FieldId::is_valid(0x00));
        assert!(!FieldId::is_valid(0x2F + 1));
    }

    #[test]
    fn test_float_conversion() {
        let value: f32 = 3.14;
        let bytes = float_to_bytes(value);
        let recovered = bytes_to_float(&bytes);
        assert!((value - recovered).abs() < f32::EPSILON);
    }

    #[test]
    fn test_int32_conversion() {
        let value: i32 = -12345;
        let bytes = int32_to_bytes(value);
        let recovered = bytes_to_int32(&bytes);
        assert_eq!(value, recovered);
    }

    #[test]
    fn test_uint32_conversion() {
        let value: u32 = 0xDEADBEEF;
        let bytes = uint32_to_bytes(value);
        let recovered = bytes_to_uint32(&bytes);
        assert_eq!(value, recovered);
    }

    #[test]
    fn test_uint16_conversion() {
        let value: u16 = 0x1234;
        let bytes = uint16_to_bytes(value);
        let recovered = bytes_to_uint16(&bytes);
        assert_eq!(value, recovered);
    }

    #[test]
    fn test_tlv_field_new() {
        let field = TLVField::new();
        assert_eq!(field.id, 0);
        assert_eq!(field.len, 0);
    }

    #[test]
    fn test_tlv_field_with_data() {
        let data = [1u8, 2, 3, 4];
        let field = TLVField::with_data(0xB0, &data);
        assert_eq!(field.id, 0xB0);
        assert_eq!(field.len, 4);
        assert_eq!(field.data[0], 1);
        assert_eq!(field.data[3], 4);
    }

    #[test]
    fn test_tlv_message_new() {
        let msg = TLVMessage::new();
        assert_eq!(msg.start_byte, START_BYTE);
        assert_eq!(msg.tlv_count, 0);
    }

    #[test]
    fn test_tlv_message_clear() {
        let mut msg = TLVMessage::with_id(0x16);
        msg.tlv_count = 3;
        msg.clear();
        assert_eq!(msg.tlv_count, 0);
        assert_eq!(msg.msg_id, 0);
    }

    #[test]
    fn test_get_msg_priority_normal() {
        assert_eq!(get_msg_priority(0x14, false), PriorityLevel::SuperCritical as u8);
        assert_eq!(get_msg_priority(0x12, false), PriorityLevel::High as u8);
        assert_eq!(get_msg_priority(0x10, false), PriorityLevel::Normal as u8);
        assert_eq!(get_msg_priority(0x16, false), PriorityLevel::Low as u8);
        assert_eq!(get_msg_priority(0x15, false), PriorityLevel::SuperLow as u8);
    }

    #[test]
    fn test_get_msg_priority_failsafe() {
        assert_eq!(get_msg_priority(0x10, true), PriorityLevel::SuperCritical as u8);
        assert_eq!(get_msg_priority(0x15, true), PriorityLevel::SuperLow as u8);
    }
}
