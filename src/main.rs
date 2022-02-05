include!(concat!(env!("OUT_DIR"), "/mod.rs"));

fn main() {
    let acc = messages::Messages::Acc(messages::AccStruct {
        field_time_us: messages::LogType::Number(0),
        field_i: messages::LogType::Number(0),
        field_sample_us: messages::LogType::Number(0),
        field_acc_x: messages::LogType::Number(0),
        field_acc_y: messages::LogType::Number(0),
        field_acc_z: messages::LogType::Number(0),
    });

    dbg!(acc);
}
