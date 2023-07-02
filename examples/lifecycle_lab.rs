
struct Reader {

}

struct Writer {

}

struct Ctx<'a> {
    reader: &'a Reader,
    writer: &'a mut Writer,
}

impl Writer {
    fn update(&mut self) {
    }
}

impl Reader {
    fn query(&self, callback: impl FnMut()) {}
}

impl Ctx<'_> {
}

fn test1(ctx: Ctx) {
    ctx.reader.query(|| {
        ctx.writer.update();
    });
    ctx.reader.query(|| {
        ctx.writer.update();
    });
}

fn main() {
    test1(Ctx { reader: &Reader {}, writer: &mut Writer {} });
}