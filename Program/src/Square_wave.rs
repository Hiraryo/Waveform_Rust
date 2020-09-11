//このコードは矩形波の音をwav形式で保存します。
//学生証番号：853806
//氏名：平井 崚太

//このサイトを参考にしました。
// http://www.kthrlab.jp/~kitahara/lecture/FY2016/opencampus/opencampus-2016-kitahara.pdf
// https://qiita.com/tan-y/items/2da4463788323e92a192
// https://achapi2718.blogspot.com/2014/03/c_18.html
// https://detail-infomation.com/rms-of-square-wave/

//このコードは、10秒間「プップップップ...」と鳴り続けてしまいます.
//波形は矩形波になっているのですが...

extern crate portaudio;
extern crate hound;

use portaudio as pa;
use std::i16;
use std::sync::mpsc;

const CHANNELS: i32 = 2;
const NUM_SECONDS: i32 = 10;
const SAMPLE_RATE: f64 = 44100.0;
const FRAMES_PER_BUFFER: u32 = 64;
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
        "PortAudio Test: output sine wave. SR = {}, BufSize = {}",
        SAMPLE_RATE, FRAMES_PER_BUFFER
    );
    
    //左耳の音量
    let mut left_phase = -1.0;
    
    //右耳の音量
    let mut right_phase = -1.0;
    
    let pa = pa::PortAudio::new()?;

    let mut settings =
        pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;
    
    settings.flags = pa::stream_flags::CLIP_OFF;
    let mut counter = 0;    //現在のバッファサイズカウント（実際はステレオだとこの数の倍）
    let mut cnt = 0;
    let (tx, rx) = mpsc::channel();

    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
        let mut idx = 0;
        let mut data: Vec<f32> = vec![];
        for _ in 0..frames {
            //指定バッファサイズより大きかったら終了
            if counter >= BUF_SIZE {
                break;
            }
            
            left_phase = left_phase;
            right_phase = right_phase;

            buffer[idx] = left_phase;
            buffer[idx + 1] = right_phase;
            //1周期の前半(0.5秒ごと)だったら、音量を-1.0にする
            if cnt >= ((BUF_SIZE as usize / NUM_SECONDS as usize) / 2 )  {
                left_phase = -1.0;
                right_phase = -1.0;
                
            }
            //1周期の後半(0.5秒ごと)だったら、音量を1.0にする
            if cnt < ((BUF_SIZE as usize / NUM_SECONDS as usize) / 2 ){
                left_phase = 1.0;
                right_phase = 1.0;
                
            }
            //1周期が終わるごとに、cntをリセット
            if cnt == (BUF_SIZE as usize / NUM_SECONDS as usize) {
                cnt = 0;
            }
            data.push(buffer[idx]);
            data.push(buffer[idx+1]);
            counter += 1;
            cnt += 1;
            idx += 2;
        }
        tx.send(data).ok(); //バッファを書き出し処理へ
        if counter >= BUF_SIZE {
            println!("音は録れてませんが、バッファから出力を行います。");
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
    let mut writer = hound::WavWriter::create("/Users/ryota/Desktop/Sound_TEST/矩形波.wav", spec).unwrap();
    stream.start()?;
    println!("Play for {} seconds.", NUM_SECONDS);
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
    println!("Test finished.");

    Ok(())
}