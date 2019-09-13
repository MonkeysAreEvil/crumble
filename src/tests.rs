#![cfg(test)]

extern crate test;

use super::{Message, Section, Header};
use test::Bencher;

fn prepare_file(filename: &str) -> String {
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::Path;

    let path = Path::new(filename);

    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(e) => panic!("Could not open plain email file: {:?}", e),
    };

    let mut email = String::new();
    match file.read_to_string(&mut email) {
        Ok(_) => {},
        Err(why) => panic!("Could not read email: {:?}", why),
    }

    email.trim().to_string()
}

fn prepare_plain() -> String {
    prepare_file("test/plain_minimal")
}

fn prepare_multipart() -> String {
    prepare_file("test/multipart_minimal")
}

fn prepare_gmail() -> String {
    // prepare_file("test/gmail")
    prepare_file("test/gmail_attachment")
}

fn prepare_bandcamp() -> String {
    prepare_file("test/bandcamp")
}

fn prepare_nested() -> String {
    prepare_file("test/nested")
}

#[test]
fn empty_string() {
    let empty_string = "";

    let email = Message::new(empty_string);
    match email {
        Ok(_) => panic!("Successfully parsed empty string"),
        Err(_e) => assert!(true) // InvalidString
    }
}

#[test]
fn bad_string() {
    let bad_string = "Hello, world!";

    let email = Message::new(bad_string);
    match email {
        Ok(_) => panic!("Successfully parsed bad string"),
        Err(_e) => assert!(true) // InvalidString
    }
}

#[bench]
fn bench_plain(b: &mut Bencher) {
    let message = prepare_plain();
    b.iter(|| Message::new(&message));
}

#[bench]
fn bench_multipart(b: &mut Bencher) {
    let message = prepare_multipart();
    b.iter(|| Message::new(&message));
}

#[bench]
fn bench_gmail(b: &mut Bencher) {
    let message = prepare_gmail();
    b.iter(|| Message::new(&message));
}

#[bench]
fn bench_bandcamp(b: &mut Bencher) {
    let message = prepare_bandcamp();
    b.iter(|| Message::new(&message));
}

#[test]
fn parse_plain() {
    let plain = prepare_plain();

    let message = Message::new(&plain);

    match message {
        Err(e) => panic!("Could not parse email: {:?}", e),
        Ok(m) => {
            let headers = m.headers;
            let sections = m.sections;

            let mut headers_reference = Vec::new();
            headers_reference.push(Header::new("Message-ID", "<0123ABCD>"));
            headers_reference.push(Header::new("Subject", "Hello, world!"));
            headers_reference.push(Header::new("Cc", "user1@example.com\nuser2@example.com"));
            headers_reference.push(Header::new("To", "user3@example.com"));
            headers_reference.push(Header::new("From", "user4@example.com"));
            headers_reference.push(Header::new("Date", "1997-07-16T19:30:30+01:00"));
            headers_reference.push(Header::new("X-Mailer", "Foo Corp Widgets 12.0.3.1.20 Build 2020040302\ntype bar\ndescription baz"));
            headers_reference.push(Header::new("X-MIMETrack", "Serialize by Foo"));
            headers_reference.push(Header::new("MIME-Version", "1.0"));
            headers_reference.push(Header::new("Content-type", "text/plain; charset=US-ASCII"));

            assert_eq!(headers.len(), headers_reference.len());
            let mut index = 0;
            for header in headers {
                assert_eq!(header.key, headers_reference[index].key.to_lowercase());
                assert_eq!(header.value, headers_reference[index].value);
                index = index + 1;
            }

            let body = String::from("Hello user3,

How is the world?
How is the moon?
How are the stars?

Cheers
user4");
            let sections_reference = vec![Section::new(&body).unwrap()];
            assert_eq!(sections.len(), sections_reference.len());
            index = 0;
            for section in sections {
                assert_eq!(section, sections_reference[index]);
                index = index + 1;
            }

        }
    }
}

#[test]
fn parse_multipart(){
    let multipart = prepare_multipart();

    let email = Message::new(&multipart);

    match email {
        Err(e) => panic!("Could not parse email: {:?}", e),
        Ok(m) => {
            let headers = m.headers;
            let sections = m.sections;

            let mut headers_reference = Vec::new();
            headers_reference.push(Header::new("From", "John Doe <example@example.com>"));
            headers_reference.push(Header::new("MIME-Version", "1.0"));
            headers_reference.push(Header::new("Content-Type", "multipart/mixed;\n    boundary='XXXXboundary text'"));

            assert_eq!(headers.len(), headers_reference.len());
            let mut index = 0;
            for header in headers {
                assert_eq!(header.key, headers_reference[index].key.to_lowercase());
                assert_eq!(header.value, headers_reference[index].value);
                index = index + 1;
            }

            let mut sections_reference = Vec::new();

            let body = String::from("content-type: text/plain\n\nthis is the body text\n\n");
            let section = Section::new(&body).unwrap();
            sections_reference.push(section);

            let body = String::from("content-type: text/plain\ncontent-disposition: attachment;\n    filename='test.txt'\n\nthis is the attachment text\n\n");
            let section = Section::new(&body).unwrap();
            sections_reference.push(section);

            assert_eq!(sections.len(), sections_reference.len());
            index = 0;
            for section in sections {
                assert_eq!(section, sections_reference[index]);
                index = index + 1;
            }
        }
    }
}

#[test]
fn parse_gmail() {
    use std::io::prelude::*;
    use std::fs::File;

    let multipart = prepare_gmail();

    let email = Message::new(&multipart);

    match email {
        Err(e) => panic!("Could not parse email: {:?}", e),
        Ok(m) => {
            let headers = m.headers;
            let sections = m.sections;

            let mut headers_reference = Vec::new();
            headers_reference.push(Header::new("return-path", "<example@gmail.com>"));
            headers_reference.push(Header::new("delivered-to", "example@example.com"));
            headers_reference.push(Header::new("received", "from mail-ed1-f43.google.com (mail-ed1-f43.google.com [209.85.208.43])
	by example.com (OpenSMTPD) with ESMTPS id ecf00d9e (TLSv1.2:ECDHE-RSA-CHACHA20-POLY1305:256:FAIL)
	for <example@example.com>;
	Tue, 10 Sep 2019 02:47:32 +0000 (UTC)"));
            headers_reference.push(Header::new("received", "by mail-ed1-f43.google.com with SMTP id y91so15364419ede.9
        for <example@example.com>; Mon, 09 Sep 2019 19:47:59 -0700 (PDT)"));
            headers_reference.push(Header::new("dkim-signature", "v=1; a=rsa-sha256; c=relaxed/relaxed;
        d=gmail.com; s=20161025;
        h=mime-version:from:date:message-id:subject:to;
        bh=lw4nVU4tXnj+HUyblPpuT7Q2zwTdNrM3vDBj+iwz9SQ=;
        b=pT2jMVu+581TAUDVdVuXaRvMdKY3QnrWOBtk9S4MacZFtbLrKwXEaxZGcoH2yl4xdF
         duzdF4CItIGPKFR4hCUIQe2Vq0mdF42Z5XzECuVkzpoE+TWQ3A45LDvuaY9yxiGVZ/g3
         ga+zQhibRirauw/zdudf5wWZx4CqQzNSY+USppi5VzvDCFbjAXeYXzzed9+8W23VWGN1
         1zYkJZyg2WbEOMO/O2eueQ1w4y+qN4j+C37HzZeAOtv/h+00tCDQVEDg92pxC22hFm+b
         sRuqOWmMvEtZ4swGH9etW75GUDaJWnHhf7yBHEsVq1EjfGLK6eVQ99JCSQxbv5z7/N+y
         Oeug=="));
            headers_reference.push(Header::new("x-google-dkim-signature", "v=1; a=rsa-sha256; c=relaxed/relaxed;
        d=1e100.net; s=20161025;
        h=x-gm-message-state:mime-version:from:date:message-id:subject:to;
        bh=lw4nVU4tXnj+HUyblPpuT7Q2zwTdNrM3vDBj+iwz9SQ=;
        b=HDw/x0yShakjbD6hNpC+uld/4fbOgtPJ3bJtlIXXrK9H4PpBiE+i5WO13yuJ6SNxAw
         AqEqTxht0eIfdNcMHtLoHZMEPCHgiceHimeZm962wVj712bQD1f6uBljPtxpthBu2c16
         9CUF+WzFRTQsGhU8gGw8xCcJmHnrzSmLgG+uAi7ZtPr5EP+3oQCxSwW2eogvZClsZqAW
         kkRwO1MIv9DuDiYXdl8skyxKp6z454gV3wCeqJTUpGv1nTj9si0el3zGpedk+wq7RTYd
         ckJWyLvoYkCfv8k8jFHBgxVuFyuGAW0+mWcF0Ot0nUl2HSL3Dhw2+g443yjUIoNqxs4n
         dxtA=="));
            headers_reference.push(Header::new("x-gm-message-state", "APjAAAWr+Sa7oXMCi+FNGGXrYdDgM4U8mqQHzcFfWiAcNgeHyRw8ztxE
	aq8d/m/G2XIOEYaVue4uEO3X5WeU1/LJWypBUrq6lbNb"));
            headers_reference.push(Header::new("x-google-smtp-source", "APXvYqzdQ+PweDlRQyYOU3o3FKIXVdUk6k3MavmnXTcuK2Ys/CHsejBMWypV70FeZutcumhAjl3oalQmXv1rS6mFwjE="));
            headers_reference.push(Header::new("x-received", "by 2002:a17:906:7e52:: with SMTP id z18mr21792389ejr.114.1568083676851;
 Mon, 09 Sep 2019 19:47:56 -0700 (PDT)"));
            headers_reference.push(Header::new("mime-version", "1.0"));
            headers_reference.push(Header::new("from", "Example <example@gmail.com>"));
            headers_reference.push(Header::new("date", "Tue, 10 Sep 2019 12:47:31 +1000"));
            headers_reference.push(Header::new("message-id", "<CAMUmi+mvKSB1x93x3su-+Yy1AvDNm2jFmgs6fVgMtGL35XuBCw@mail.gmail.com>"));
            headers_reference.push(Header::new("subject", "Example"));
            headers_reference.push(Header::new("to", "example@example.com"));
            headers_reference.push(Header::new("content-type", r#"multipart/mixed; boundary="0000000000008a01e4059229eec0""#));

            assert_eq!(headers.len(), headers_reference.len());
            let mut index = 0;
            for header in headers {
                assert_eq!(header.key, headers_reference[index].key.to_lowercase());
                assert_eq!(header.value, headers_reference[index].value);
                index = index + 1;
            }

            let mut sections_reference = Vec::new();

            let section = Section::Multipart {
                headers: vec![Header::new("content-type", r#"multipart/alternative; boundary="0000000000008a01e1059229eebe""#)],
                body: vec![
                    Box::new(Section::Multipart {
                        headers: vec![Header::new("content-type", r#"text/plain; charset="UTF-8""#)],
                        body: vec![Box::new(Section::Plain {body: String::from("Hello, world!\n\n").as_bytes().to_vec()})]
                    }),
                    Box::new(Section::Multipart {
                        headers: vec![Header::new("content-type", r#"text/html; charset="UTF-8""#)],
                        body: vec![Box::new(Section::Plain {body: String::from(r#"<div dir="ltr">Hello, world!<br></div>

"#).as_bytes().to_vec()})]
                    })
                ]
            };

            sections_reference.push(section);

            let body = r#"content-type: image/png; name="Lenna_(test_image).png"
content-disposition: attachment; filename="Lenna_(test_image).png"
content-transfer-encoding: base64
content-id: <f_k0d8idqy0>
x-attachment-id: f_k0d8idqy0"#;
            let mut f = File::open("test/Lenna_(test_image).base64").unwrap();
            let mut tmp = Vec::new();
            f.read_to_end(&mut tmp).unwrap();
            let tmp = String::from_utf8_lossy(&tmp);
            let body = format!("{}\n\n{}", body, tmp);
            let section = Section::new(&body).unwrap();
            sections_reference.push(section);

            assert_eq!(sections.len(), sections_reference.len());
            index = 0;
            for section in sections {
                assert_eq!(section, sections_reference[index]);
                index = index + 1;
            }
        }
    }
}

#[test]
fn parse_bandcamp(){
    let multipart = prepare_bandcamp();

    let email = Message::new(&multipart);

    match email {
        Err(e) => panic!("Could not parse email: {:?}", e),
        Ok(m) => {
            let headers = m.headers;
            let sections = m.sections;

            let mut headers_reference = Vec::new();
            headers_reference.push(Header::new("return-path", "<bounces+715912-0519-example=example.com@email.bandcamp.com>"));
            headers_reference.push(Header::new("delivered-to", "example@example.com"));
            headers_reference.push(Header::new("received", "from o3.email.bandcamp.com (o3.email.bandcamp.com [198.21.0.215])
\tby example.com (OpenSMTPD) with ESMTPS id a1fa99f4 (TLSv1.2:ECDHE-RSA-AES256-GCM-SHA384:256:NO)
\tfor <example@example.com>;
\tSun, 1 Sep 2019 18:47:11 +0000 (UTC)"));
            headers_reference.push(Header::new("dkim-signature", "v=1; a=rsa-sha1; c=relaxed/relaxed; 
\td=email.bandcamp.com; 
\th=from:to:subject:reply-to:mime-version:content-type; s=smtpapi; 
\tbh=GMq8o6onqHwkbKkFgqnrnpHezjc=; b=zdXYpJZGtSuUMv2xRDL3DhRYrQwXZ
\tU8crpl/b+TLRc+h/GZcddBH1Mw6kg+FAs5Nuy1npOE7d3zXACBDE95hya/RkcF8Q
\tya7fvewMlGBBfw8ZFRhukaDnTYc9GdyWn/rd5K33a8g7QdlfVpeL++x5sFAcyfVO
\tVkPI6RaTPa89DM="));
            headers_reference.push(Header::new("received", "by filter0116p1iad2.sendgrid.net with SMTP id filter0116p1iad2-543-5D6C1235-13
        2019-09-01 18:47:17.604755642 +0000 UTC m=+429561.889968037"));
            headers_reference.push(Header::new("received", "from bandcamp.com (ef.82.3da9.ip4.static.sl-reverse.com [169.61.130.239])
	by ismtpd0009p1sjc2.sendgrid.net (SG) with ESMTP id K7NdDks5Rp-hj6zekhaxVQ
	for <example@example.com>; Sun, 01 Sep 2019 18:47:16.923 +0000 (UTC)"));
            headers_reference.push(Header::new("from", "Bandcamp <noreply@bandcamp.com>"));
            headers_reference.push(Header::new("to", "example <example@example.com>"));
            headers_reference.push(Header::new("subject", "=?UTF-8?B?TmV3IGZyb20gTWFsb2thcnBhdGFuOiAiU3RyaWTFvmllIGRuaSIgcmVk?="));
            headers_reference.push(Header::new("reply-to", "<noreply@bandcamp.com>"));
            headers_reference.push(Header::new("mime-version", "1.0"));
            headers_reference.push(Header::new("content-type", r#"multipart/alternative;
 boundary="it_was_only_a_kiss""#));
            headers_reference.push(Header::new("message-id", "<K7NdDks5Rp-hj6zekhaxVQ@ismtpd0009p1sjc2.sendgrid.net>"));
            headers_reference.push(Header::new("date", "Sun, 01 Sep 2019 18:47:17 +0000 (UTC)"));
            headers_reference.push(Header::new("x-sg-eid", "nBOqntU0yBFjPVlNdjQaY3wDu4yTqLEvn1WO8Aw6GMB2LHOJyurZxQWDoS1ERDO7yQvQFeG32M8BCi
 laaSf8u02bkFHbT0Xv9H7gaVogCrkLtZogborUCORVUiPJGhw1UI+m13mfwglDAtiJrT1f96VpGs2T
 wwX20QhrGjGO2pMraVi2fI6k33Jlzv+pQ6HNut2ksNDg06CgBBC6mnB3KA==

This is a multi-part message in MIME format."));

            assert_eq!(headers.len(), headers_reference.len());
            let mut index = 0;
            for header in headers {
                assert_eq!(header.key, headers_reference[index].key.to_lowercase());
                assert_eq!(header.value, headers_reference[index].value);
                index = index + 1;
            }

            let mut sections_reference = Vec::new();

            let body = String::from(r#"content-type: text/plain; charset=utf-8
content-transfer-encoding: quoted-printable

=0D
Greetings example,=0D
=0D
Malokarpatan just added "Strid=C5=BEie dni" red to Bandcamp, check it out a=
t:=0D
=0D
http://malokarpatan.bandcamp.com/merch/strid-ie-dni-red?from=3Dfanpub_fb_me=
rch=0D
=0D
=0D
=0D
=0D
Enjoy!=0D
=0D
=0D
Unfollow Malokarpatan by visiting:=0D
=0D
http://malokarpatan.bandcamp.com/fan_unsubscribe?band_id=3D1884471442&email=
=3Dexample%40example.com&fan_id=3Dxxxxxx&sig=3Dxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx=
2=0D
=0D
=0D


"#);
            let section = Section::new(&body).unwrap();
            sections_reference.push(section);

            let body = String::from(r##"content-type: text/html; charset=utf-8
content-transfer-encoding: quoted-printable

<div id=3D"msg" style=3D"color:#595959;font-family: 'Helvetica Neue',arial,=
verdana,sans-serif;line-height:150%;padding:0;font-size:14px">=0D
=0D
<div style=3D"width:210px;min-height:158px;margin-bottom: 20px;">=0D
    <a href=3D"http://malokarpatan.bandcamp.com/merch/strid-ie-dni-red?from=
=3Dfanpub_fb_merch">=0D
        <img style=3D"width:210px;min-height:158px;" src=3D"http://f0.bcbit=
s.com/img/0017234612_36.jpg" alt=3D"&quot;Strid=C5=BEie dni&quot; red Art">=
=0D
    </a>=0D
</div>=0D
=0D
Greetings example,=0D
<br>=0D
Malokarpatan just added <span style=3D"font-style: italic;">&quot;Strid=C5=
=BEie dni&quot; red</span> to Bandcamp, <a href=3D"http://malokarpatan.band=
camp.com/merch/strid-ie-dni-red?from=3Dfanpub_fb_merch" style=3D"color:#068=
7f5;text-decoration:none;">check it out here</a>.=0D
=0D
=0D
<br><br>=0D
Enjoy!=0D
<br><br>=0D
<a href=3D"http://bandcamp.com"><img src=3D"http://bandcamp.com/img/email/b=
c-logo-small-2.gif" width=3D"105" height=3D"19" border=3D"0" alt=3D"bandcam=
p logo"></a><br/>=0D
<br>=0D
<span style=3D"font-size:11px;border-top:1px dotted #ccc;width:95%;display:=
block;padding:1em 0;margin:1em 0 0;"><a style=3D"color:#999;text-decoration=
:none;font-size:11px;" href=3D"http://malokarpatan.bandcamp.com/fan_unsubsc=
ribe?band_id=3D1884471442&amp;email=3Dexample%40example.com&amp;fan_id=3D27541=
1&amp;sig=3D63babede6c4610d722c7828045462822">Unfollow Malokarpatan</a></sp=
an>=0D
<br>&nbsp;=0D
</div>=0D


"##);
            let section = Section::new(&body).unwrap();
            sections_reference.push(section);

            assert_eq!(sections.len(), sections_reference.len());
            index = 0;
            for section in sections {
                assert_eq!(section, sections_reference[index]);
                index = index + 1;
            }
        }
    }
}

#[test]
fn parse_nested() {
    let nested = prepare_nested();

    let message = Message::new(&nested);

    match message {
        Err(e) => panic!("Could not parse email: {:?}", e),
        Ok(m) => {
            let headers = m.headers;
            let sections = m.sections;

            let mut headers_reference = Vec::new();
            headers_reference.push(Header::new("Return-Path", "<example@gmail.com>"));
            headers_reference.push(Header::new("Delivered-To", "example@example.com"));
            headers_reference.push(Header::new("MIME-Version", "1.0"));
            headers_reference.push(Header::new("From", "Example <example@gmail.com>"));
            headers_reference.push(Header::new("Date", "Tue, 10 Sep 2019 12:47:31 +1000"));
            headers_reference.push(Header::new("Message-ID", "<CAMUmi+mvKSB1x93x3su-+Yy1AvDNm2jFmgs6fVgMtGL35XuBCw@mail.gmail.com>"));
            headers_reference.push(Header::new("Subject", "Example"));
            headers_reference.push(Header::new("To", "example@example.com"));
            headers_reference.push(Header::new("Content-Type", r#"multipart/mixed; boundary="boundary_A""#));

            assert_eq!(headers.len(), headers_reference.len());
            let mut index = 0;
            for header in headers {
                assert_eq!(header.key, headers_reference[index].key.to_lowercase());
                assert_eq!(header.value, headers_reference[index].value);
                index = index + 1;
            }

            let mut sections_reference = Vec::new();

            let section = Section::Multipart {
                headers: vec![Header::new("content-type", r#"multipart/alternative; boundary="boundary_B"

Level A"#)],
                body: vec![
                    Box::new(Section::Multipart {
                        headers: vec![Header::new("content-type", r#"multipart/alternative; boundary="boundary_C1"

Level B1"#)],
                        body: vec![
                            Box::new(Section::Multipart {
                                headers: vec![Header::new("content-type", r#"multipart/alternative; boundary="boundary_D1"

Level C1"#)],
                                body: vec![
                                    Box::new(Section::Multipart {
                                        headers: vec![Header::new("content-type", r#"text/plain; charset="UTF-8""#)],
                                        body: vec![
                                            Box::new(Section::Plain{body: "Level D1\n\n".as_bytes().to_vec()})
                                        ]
                                    })
                                ]

                            })
                        ]
                    }),
                    Box::new(Section::Multipart {
                        headers: vec![Header::new("content-type", r#"multipart/alternative; boundary="boundary_C2"

Level B2"#)],
                        body: vec![
                            Box::new(Section::Multipart {
                                headers: vec![Header::new("content-type", r#"text/plain; charset="UTF-8""#)],
                                body: vec![
                                    Box::new(Section::Plain{body: "Level C2\n\n".as_bytes().to_vec()})
                                ]

                            })
                        ]
                    })
                ]
            };

            sections_reference.push(section);

            assert_eq!(sections.len(), sections_reference.len());
            index = 0;
            for section in sections {
                assert_eq!(section, sections_reference[index]);
                index = index + 1;
            }

        }
    }
}
