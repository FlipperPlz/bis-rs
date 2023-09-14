
//----------------------------------IO--------------------------------------------------------------
mod io {
    pub trait BinaryObject<BinarizationOptions, DebinarizationOptions> :
        Binarizable<BinarizationOptions> + Debinarizable<DebinarizationOptions>{}

    pub trait StrictBinaryObject<BinarizationOptions, DebinarizationOptions, ValidationOptions> :
        BinaryObject<BinarizationOptions, DebinarizationOptions> + Validatable<ValidationOptions>{}
    pub trait Debinarizable<DebinarizationOptions> : Sized {
        fn debinarize (
            reader: &mut impl std::io::Read,
            options: &Self::DebinarizationOptions
        ) -> Result<*Self, Box<dyn std::error::Error>>;
    }

    pub trait Binarizable<BinarizationOptions> {
        fn binarize (
            &self,
            writer: &mut impl std::io::Write,
            options: BinarizationOptions
        ) -> Result<(), Box<dyn std::error::Error>>;
    }

    pub trait Validatable<ValidationOptions> {

        fn validate (
            &self,
            options: ValidationOptions
        ) -> Result<(), Box<dyn std::error::Error>>;
    }
}
//----------------------------------Compression-----------------------------------------------------
mod compression {
    mod lzss {
        const N:         usize = 4096;
        const F:         usize = 18;
        const NIL:       usize = N;
        const FILL:      u8    = 0x20;
        const THRESHOLD: usize = 2;

        #[derive(Debug, Default)]
        struct Context {
            match_position:    usize,
            match_length:      usize,
            previous_children: [usize; N + 1],
            next_children:     [usize; N + 257],
            parents:           [usize; N + 1],
            buffer:            [u8;    N + F - 1]
        }
        pub fn encode(
            input: &[u8]
        ) -> Result<Vec<u8>, dyn std::error::Error> {
            let mut compressed_buffer: Vec<u8> = vec![];
            let mut context = Context::new_encoder();

            let mut len: usize = 0;
            let mut mask: u8 = 1;
            let mut input_idx: usize = 0;
            let mut code_idx: usize = mask as usize;
            let mut code_size: u32 = 0;
            let mut s: usize = 0;
            let mut r = N - F;
            let stop_pos = input.len();
            
            let mut code: [u8; 17] = [0; 17];
            for i in s..r { context.buffer[i] = FILL }
            while len < F && input_idx < stop_pos {
                context.buffer[r + len] = input[input_idx];
                input_idx += 1; len += 1;
            }
            for i in 1..=F { context.insert_node(r - i) }
            context.insert_node(r);
            loop {
                if context.match_length > len {
                    context.match_length = len;
                }

                if context.match_length <= THRESHOLD {
                    context.match_length = 1;
                    code[0] |= mask;
                    code[code_idx] = context.buffer[r];
                    code_idx += 1;
                } else {
                    let encoded_position = (r - context.match_position) & (N - 1);
                    code[code_idx] = encoded_position as u8;
                    code_idx += 1;
                    code[code_idx] = (
                        ((encoded_position >> 3) & 0xf0) |
                        (context.match_length - (THRESHOLD + 1))
                    ) as u8;
                    code_idx += 1;
                }

                mask = mask << 1;
                if mask == 0 {
                    compressed_buffer.extend_from_slice(&code[0..code_idx]);
                    code[0] = 0;
                    code_idx = 1;
                    mask = 1;
                }

                let last_match_length = context.match_length;
                for _i in 0..std::cmp::min(last_match_length, stop_pos - input_idx) {
                    context.delete_node(s);
                    let c = input[input_idx];
                    input_idx += 1;
                    context.buffer[s] = c;
                    if s < F - 1 { context.buffer[s + N] = c }
                    s = (s + 1) & (N - 1);
                    r = (r + 1) & (N - 1);
                    context.insert_node(r);
                }
                for _i in 0..last_match_length {
                    context.delete_node(s);
                    s = (s + 1) & (N - 1);
                    r = (r + 1) & (N - 1);
                    len -= 1;
                    if len != 0 { context.insert_node(r) }
                }

                if len == 0 {
                    break;
                }
            }

            if code_idx > 1 {
                compressed_buffer.extend_from_slice(&code[0..code_idx]);
                code_size += code_idx
            }

            Ok(compressed_buffer)
        }

        impl Context {


            fn insert_node(&mut self, node: usize) {
                let mut cmp = 1;
                self.match_length = 0;
                let mut p = N + 1 + self.buffer[node] as usize;
                loop {
                    if cmp >= 0 {
                        if self.next_children == NIL {
                            self.next_children[p] = node;
                            self.parents[node] = p;
                            return
                        }

                        p = self.next_children[p]
                    } else {
                        if self.previous_children[p] == NIL {
                            self.previous_children[p] = node;
                            self.parents[node] = p;
                            return;
                        }

                        p = self.previous_children[p];
                    }

                    let i = (1..F)
                        .find(
                            |&i| {
                                cmp = self.buffer[node + i] as isize - self.buffer[p + i] as isize;

                                cmp != 0
                            }
                        )
                        .unwrap_or(F);

                    if i > self.match_length {
                        self.match_position = p;
                        self.match_length = i;
                        if self.match_length >= F { break }
                    }
                }

                self.parents[node] = self.parents[p];
                self.previous_children[node] = self.previous_children[p];
                self.next_children[node] = self.next_children[p];
                self.parents[self.previous_children[p]] = node;
                self.parents[self.next_children[p]] = node;

                if self.next_children[self.parents[p]] == p {
                    self.next_children[self.parents[p]] = node
                } else {
                    self.previous_children[self.parents[p]] = node
                }

                self.parents[p] = NIL;
            }

            fn delete_node(&mut self, node: usize) {
                if self.parents[node] == NIL {
                    return;
                }

                let q = if self.next_children[node] == NIL {
                    self.previous_children[node]
                } else if self.previous_children[node] == NIL {
                    self.next_children[node]
                } else {
                    let mut q = self.previous_children[node];
                    if self.next_children[q] !=  NIL {
                        while self.next_children[q] != NIL {
                            q = self.next_children[q]
                        }

                        self.next_children[self.parents[q]] = self.previous_children[q];
                        self.parents[self.previous_children[q]] = self.parents[q];
                        self.previous_children[q] = self.previous_children[node];
                        self.parents[self.previous_children[node]] = q;
                    }

                    self.next_children[q] = self.next_children[node];
                    self.parents[self.next_children[node]] = q;

                    q
                };

                self.parents[q] = self.parents[node];

                if self.next_children[self.parents[node]] == node {
                    self.next_children[self.parents[node]] = q
                } else {
                    self.previous_children[self.parents[node]] = q;
                }

                self.parents[node] = NIL;
            }

            fn new_encoder() -> Self {
                Context {
                    match_position: 0,
                    match_length: 0,
                    previous_children: [NIL; N + 1],
                    next_children: [NIL; N + 257],
                    parents: [NIL; N + 1],
                    buffer: [FILL; N + F - 1],
                }
            }

        }


        //
        // pub fn decode(input: &[u8], length: u32) -> (Vec<u8>, Box<dyn std::error::Error>) {
        //     let mut decompressed_buffer: Vec<u8> = vec![0u8; length as usize];
        // }

    }

}
