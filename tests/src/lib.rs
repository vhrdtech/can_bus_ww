use wire_weaver::wire_weaver_api;

#[wire_weaver_api(
    ww = "../vb135_fdcan_usb.ww",
    api_model = "client_server_v0_1",
    skip_api_model_codegen = true,
    no_alloc = false,
    // derive = "Debug, PartialEq, Eq"
)]
mod api {}

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
        let frame = CanFrame {
            timestamp_us: 0xaa000011,
            id: CanId::Standard(0xbb),
            kind: CanFrameKind::Classic { rtr: false },
            data: vec![0xcc, 0xdd, 0xee, 0xff],
        };
        check_frame(&frame, &hex!("110000aa 0 0 bb00 0 0 ccddeeff 04")[..]);
    }

    #[test]
    fn can_frame_classic_extended_format_sanity() {
        let frame = CanFrame {
            timestamp_us: 0xaa,
            id: CanId::Extended(0xbbccdd),
            kind: CanFrameKind::Classic { rtr: false },
            data: vec![0xcc, 0xdd, 0xee, 0xff],
        };
        check_frame(&frame, &hex!("aa000000 1 0 ddccbb00 0 0 ccddeeff 04")[..]);
    }

    #[test]
    fn can_frame_fd_standard_format_sanity() {
        let frame = CanFrame {
            timestamp_us: 0xaa,
            id: CanId::Standard(0xbb),
            kind: CanFrameKind::Fd {
                brs: true,
                esi: true,
            },
            data: vec![0xcc, 0xdd, 0xee, 0xff],
        };
        check_frame(&frame, &hex!("aa000000 0 0 bb00 1 c ccddeeff 04")[..]);
    }

    #[test]
    fn can_frame_fd_extended_format_sanity() {
        let frame = CanFrame {
            timestamp_us: 0xaa,
            id: CanId::Extended(0xbbccdd),
            kind: CanFrameKind::Fd {
                brs: false,
                esi: false,
            },
            data: vec![0xcc, 0xdd, 0xee, 0xff],
        };
        check_frame(&frame, &hex!("aa000000 1 0 ddccbb00 1 0 ccddeeff 04")[..]);
    }
}
