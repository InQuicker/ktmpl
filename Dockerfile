FROM debian:jessie

RUN apt-get update -qq && apt-get install wget curl git file gcc -qqy

RUN wget https://static.rust-lang.org/rustup.sh && chmod +x rustup.sh && ./rustup.sh --disable-sudo --channel=nightly

RUN git clone https://github.com/InQuicker/ktmpl && cd ktmpl && cargo install --path .

RUN cp /root/.cargo/bin/ktmpl /bin

ENTRYPOINT [ "/bin/ktmpl" ]
