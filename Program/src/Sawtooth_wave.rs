//このコードはノコギリ波の音をwav形式で保存します。
//学生証番号：853806
//氏名：平井 崚太

//このサイトを参考にしました。
// https://github.com/RustAudio/rust-portaudio/blob/master/examples/saw.rs
// https://qiita.com/kamiro/items/5493dd109b7cc5043814
// http://www.kthrlab.jp/~kitahara/lecture/FY2016/opencampus/opencampus-2016-kitahara.pdf
// https://qiita.com/tan-y/items/2da4463788323e92a192
// https://achapi2718.blogspot.com/2014/03/c.html


extern crate portaudio;
extern crate hound;

use portaudio as pa;
use std::i16;
use std::sync::mpsc;

const CHANNELS: i32 = 2;    //チャンネル数(1->モノラル、2->ステレオ)
const NUM_SECONDS: i32 = 10; //録音秒数
const SAMPLE_RATE: f64 = 44_100.0;  //44100hz
const FRAMES_PER_BUFFER: u32 = 64;  //フレームごとのバッファ数
const BITS_PER_SAMPLE: u16 = 16; //量子化ビット数
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
        "PortAudio Test: output sawtooth wave. SR = {}, BufSize = {}",
        SAMPLE_RATE, FRAMES_PER_BUFFER
    );

    let mut left_saw = 0.0;
    let mut right_saw = 0.0;

    let pa = pa::PortAudio::new()?;

    let mut settings =
        pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;
    
    settings.flags = pa::stream_flags::CLIP_OFF;
    let mut counter = 0;    //現在のバッファサイズカウント
    let (tx, rx) = mpsc::channel();

    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
        let mut idx = 0;
        let mut data: Vec<f32> = vec![];

        //0からフレーム数までループ
        for _ in 0..frames {
            //10秒経過すると、ループ終了
            if counter >= BUF_SIZE {
                break;
            }

            buffer[idx] = left_saw;
            buffer[idx + 1] = right_saw;
            left_saw += 0.01;
            if left_saw >= 1.0 {
                left_saw -= 2.0;
            }
            right_saw += 0.01;
            if right_saw >= 1.0 {
                right_saw -= 2.0;
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
    let mut writer = hound::WavWriter::create("/Users/ryota/Desktop/Sound_TEST/ノコギリ波.wav", spec).unwrap();
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