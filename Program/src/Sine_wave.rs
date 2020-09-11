//このコードは正弦波の音をwav形式で保存します。
//学生証番号：853806
//氏名：平井 崚太

//このサイトを参考にしました。
// https://github.com/RustAudio/rust-portaudio/blob/master/examples/sine.rs
// https://qiita.com/kamiro/items/4d61f5ee621c25c2526a
// https://qiita.com/kamiro/items/5493dd109b7cc5043814
// http://www.kthrlab.jp/~kitahara/lecture/FY2016/opencampus/opencampus-2016-kitahara.pdf
// https://qiita.com/ohisama@github/items/9b82595f65579c7e06e2
// https://qiita.com/tan-y/items/2da4463788323e92a192



extern crate portaudio;
extern crate hound;

use portaudio as pa;
use std::f64::consts::PI;
use std::sync::mpsc;

const CHANNELS: i32 = 2;
const NUM_SECONDS: i32 = 10;
const SAMPLE_RATE: f64 = 44100.0;
const FRAMES_PER_BUFFER: u32 = 64;
const BITS_PER_SAMPLE: u16 = 16; //量子化ビット数
const TABLE_SIZE: usize = 440;  //1周期の周波数
const BUF_SIZE: usize = SAMPLE_RATE as usize * NUM_SECONDS as usize;    //保存するバッファサイズ

fn main() {
    match run() {
        Ok(_) => {}
        e => {
            eprintln!("Example failed with the following: {:?}", e);
        }
    }
}

fn run() -> Result<(), pa::Error> {
    println!(
        "PortAudio Test: output sine wave. SR = {}, BufSize = {}",
        SAMPLE_RATE, FRAMES_PER_BUFFER
    );

    // 正弦波の周波数を0にする
    let mut sine = [0.0; TABLE_SIZE];
    
    for i in 0..TABLE_SIZE {
        sine[i] = (i as f64 / TABLE_SIZE as f64 * PI * 2.0).sin() as f32;
    }
    //左耳の音量
    let mut left_phase = 0;
    //右耳の音量
    let mut right_phase = 0;

    let pa = pa::PortAudio::new()?;

    let mut settings = pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;
    settings.flags = pa::stream_flags::CLIP_OFF;
    let mut counter = 0;    //現在のバッファサイズカウント（実際はステレオだとこの数の倍）
    let (tx, rx) = mpsc::channel();

    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
        let mut idx = 0;
        let mut data: Vec<f32> = vec![];
        for _ in 0..frames {
            //指定バッファサイズより大きかったら終了
            if counter >= BUF_SIZE {
                break;
            }
            buffer[idx] = sine[left_phase];
            buffer[idx + 1] = sine[right_phase];
            left_phase += 1;
            if left_phase >= TABLE_SIZE {
                left_phase -= TABLE_SIZE;
            }
            right_phase += 1;
            if right_phase >= TABLE_SIZE {
                right_phase -= TABLE_SIZE;
            }
            data.push(buffer[idx]);
            data.push(buffer[idx+1]);
            counter += 1;
            idx += 2;
        }
        tx.send(data).ok(); //wavファイルへ書き出す処理へ
        if counter >= BUF_SIZE {
            println!("録音ができました。バッファから出力を行います。");
            pa::Complete
        }else{
            pa::Continue
        }
    };

    let mut stream = pa.open_non_blocking_stream(settings, callback)?;

    //wav書き出し設定
    let spec = hound::WavSpec {
        channels: CHANNELS as u16,  //チャンネル数
        sample_rate: SAMPLE_RATE as u32,    //サンプリングレート
        bits_per_sample: BITS_PER_SAMPLE,   //量子化ビット数
        sample_format: hound::SampleFormat::Int,    //インテジャーPCM
    };
    let mut writer = hound::WavWriter::create("/Users/ryota/Desktop/Sound_TEST/正弦波.wav", spec).unwrap();
    stream.start()?;
    println!("この音は {} 秒間再生され、その後指定したパスにwav形式で保存されます。", NUM_SECONDS);
    pa.sleep(NUM_SECONDS * 1_000);

    while let true = stream.is_active()? {
        while let Ok(data) = rx.try_recv() {
            //bufferをファイルに書き出し
            for d in &data {
                let amplitude = i16::MAX as f32;    //振れ幅（音量）を調整
                writer.write_sample((d * amplitude) as i16).unwrap();   //書き出し
             }
        }
    }
    stream.stop()?;
    stream.close()?;
    writer.finalize().unwrap();
    println!("保存が完了しました。");

    Ok(())
}