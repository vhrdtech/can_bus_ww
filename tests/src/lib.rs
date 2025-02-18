use wire_weaver::wire_weaver_api;

#[wire_weaver_api(
    ww = "../vb135_fdcan_usb.ww",
    api_model = "client_server_v0_1",
    server = true,
    no_alloc = false,
    derive = "Debug, PartialEq, Eq",
    debug_to_file = "./target/ww.rs"
)]
mod api {
    #[allow(dead_code)]
    struct Context {}

    impl Context {
        #[allow(dead_code)]
        async fn termination(&mut self, _enabled: bool) {}
    }
}

#[cfg(test)]
mod tests {
    use crate::api::*;
    use hex_literal::hex;
    use wire_weaver::shrink_wrap::{
        BufReader, BufWriter, DeserializeShrinkWrap, ElementSize, SerializeShrinkWrap,
    };

    fn check_frame(frame: &CanFrame, bytes: &[u8]) {
        let mut buf = [0u8; 128];
        let mut wr = BufWriter::new(&mut buf);
        frame.ser_shrink_wrap(&mut wr).unwrap();
        let buf = wr.finish_and_take().unwrap();
        // println!("{:02x?}", &buf);
        assert_eq!(buf, bytes);

        let mut rd = BufReader::new(buf);
        let frame_des = CanFrame::des_shrink_wrap(&mut rd, ElementSize::Implied).unwrap();
        assert_eq!(&frame_des, frame);
        // println!("{:02x?}", frame_des);
    }

    #[test]
    fn can_frame_classic_standard_format_sanity() {
        let mut frame = CanFrame {
            id: CanId::Standard(0xbb),
            kind: CanFrameKind::Classic { rtr: false },
            data: vec![0xcc, 0xdd, 0xee, 0xff],
            timestamp_us: Some(0xaa000011),
        };
        check_frame(&frame, &hex!("0 0 bb00 0 4 ccddeeff 110000aa 04")[..]);

        frame.kind = CanFrameKind::Classic { rtr: true };
        check_frame(&frame, &hex!("0 0 bb00 0 c ccddeeff 110000aa 04")[..]);

        frame.timestamp_us = None;
        check_frame(&frame, &hex!("0 0 bb00 0 8 ccddeeff 04")[..]);
    }

    #[test]
    fn can_frame_classic_extended_format_sanity() {
        let mut frame = CanFrame {
            id: CanId::Extended(0xbbccdd),
            kind: CanFrameKind::Classic { rtr: false },
            data: vec![0xcc, 0xdd, 0xee, 0xff],
            timestamp_us: Some(0xaa),
        };
        check_frame(&frame, &hex!("1 0 ddccbb00 0 4 ccddeeff aa000000 04")[..]);

        frame.timestamp_us = None;
        check_frame(&frame, &hex!("1 0 ddccbb00 0 0 ccddeeff 04")[..]);
    }

    #[test]
    fn can_frame_fd_standard_format_sanity() {
        let mut frame = CanFrame {
            id: CanId::Standard(0xbb),
            kind: CanFrameKind::Fd {
                brs: true,
                esi: true,
            },
            data: vec![0xcc, 0xdd, 0xee, 0xff],
            timestamp_us: Some(0xaa),
        };
        check_frame(&frame, &hex!("0 0 bb00 1 e ccddeeff aa000000 04")[..]);

        frame.timestamp_us = None;
        check_frame(&frame, &hex!("0 0 bb00 1 c ccddeeff 04")[..]);
    }

    #[test]
    fn can_frame_fd_extended_format_sanity() {
        let mut frame = CanFrame {
            id: CanId::Extended(0xbbccdd),
            kind: CanFrameKind::Fd {
                brs: false,
                esi: false,
            },
            data: vec![0xcc, 0xdd, 0xee, 0xff],
            timestamp_us: Some(0xaa),
        };
        check_frame(&frame, &hex!("1 0 ddccbb00 1 2 ccddeeff aa000000 04")[..]);

        frame.timestamp_us = None;
        check_frame(&frame, &hex!("1 0 ddccbb00 1 0 ccddeeff 04")[..]);
    }

    #[test]
    fn can_frame_in_stream_update_event() {
        let frame = CanFrame {
            id: CanId::Extended(0xbbccdd),
            kind: CanFrameKind::Fd {
                brs: false,
                esi: false,
            },
            data: vec![0xcc, 0xdd, 0xee, 0xff],
            timestamp_us: Some(0xaa),
        };
        let mut frame_scratch = [0u8; 128];
        let mut event_scratch = [0u8; 128];
        let bytes = canbus_stream_ser(&frame, &mut frame_scratch, &mut event_scratch).unwrap();
        // println!("{bytes:02x?}");
        // event.seq is_ok=1 repr(EventKind)=5=StreamUpdate path=[1] free(0) frame=15B free(0) Nib16Rev=15(payload len) Nib16Rev=1(path len)
        assert_eq!(bytes, hex!("0000 8 5 1 0 10ddccbb0012ccddeeffaa00000004 0 79 1"))
    }
}
