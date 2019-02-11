extern crate gstreamer as gst;
use gst::prelude::*;
extern crate glib;

fn sink_to_kvs() {
    // Initialize GStreamer
    gst::init().unwrap();

    // Create the elements
    let source = gst::ElementFactory::make("v4l2src", "source").unwrap();
    let video_convert = gst::ElementFactory::make("videoconvert", "video_convert").unwrap();
    let raw_filter = gst::ElementFactory::make("capsfilter", "raw_filter").unwrap();
    let omxh264enc_convert = gst::ElementFactory::make("omxh264enc", "omxh264enc_convert").unwrap();
    let h264parse_convert = gst::ElementFactory::make("h264parse", "h264parse_convert").unwrap();
    let encode_filter = gst::ElementFactory::make("capsfilter", "encoder_filter").unwrap();
    let sink = gst::ElementFactory::make("kvssink", "sink").unwrap();

    // Create the empty pipeline
    let pipeline = gst::Pipeline::new("kvs-pipeline");

    //link elements
    pipeline
        .add_many(&[
            &source,
            &video_convert,
            &raw_filter,
            &omxh264enc_convert,
            &h264parse_convert,
            &encode_filter,
            &sink,
        ])
        .unwrap();
    source
        .link(&video_convert)
        .expect("Elements could not be linked.");
    video_convert
        .link(&raw_filter)
        .expect("Elements could not be linked.");
    raw_filter
        .link(&omxh264enc_convert)
        .expect("Elements could not be linked.");
    omxh264enc_convert
        .link(&h264parse_convert)
        .expect("Elements could not be linked.");
    h264parse_convert
        .link(&encode_filter)
        .expect("Elements could not be linked.");
    encode_filter
        .link(&sink)
        .expect("Elements could not be linked.");

    // Set the properties
    sink.set_property_from_str("stream-name", "my-stream");
    sink.set_property_from_str("frame-timestamp", "dts-only");
    sink.set_property_from_str("access-key", "my-access-key");
    sink.set_property_from_str("secret-key", "my-secret-key");
    sink.set_property_from_str("aws-region", "ap-northeast-1");

    source.set_property_from_str("device", "/dev/video0");
    source.set_property_from_str("do-timestamp", "TRUE");

    raw_filter.set_property_from_str("caps", "video/x-raw");
    raw_filter.set_property_from_str("format", "I420");

    let query_caps_h264 = gst::Caps::new_simple("video/x-h264", &[("stream-format", &"avc")]);
    encode_filter
        .set_property("caps", &query_caps_h264)
        .unwrap();

    // Start playing
    //.expect("Unable to set the pipeline to the `Playing` state");

    let main_loop = glib::MainLoop::new(None, false);
    let main_loop_clone = main_loop.clone();

    let bus = pipeline.get_bus().unwrap();
    bus.connect_message(move |_, msg| match msg.view() {
        gst::MessageView::Error(err) => {
            let main_loop = &main_loop_clone;
            eprintln!(
                "Error received from element {:?}: {}",
                err.get_src().map(|s| s.get_path_string()),
                err.get_error()
            );
            eprintln!("Debugging information: {:?}", err.get_debug());
            main_loop.quit();
        }
        _ => (),
    });
    bus.add_signal_watch();

    pipeline
        .set_state(gst::State::Playing)
        .into_result()
        .unwrap();

    main_loop.run();

    pipeline.set_state(gst::State::Null).into_result().unwrap();

    bus.remove_signal_watch();
}

fn main() {
    sink_to_kvs();
}
