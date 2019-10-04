use crate::atom;
use crate::bif;
use crate::bitstring;
use crate::exception::{Exception, Reason};
use crate::process::RcProcess;
use crate::value::{self, CastFrom, CastInto, Cons, Term, Tuple, Variant};
use crate::vm::Machine;
use lexical;

pub fn md5_1(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let bytes = list_to_iodata(args[0])?;

    let heap = &process.context_mut().heap;
    let digest = md5::compute(bytes);
    Ok(Term::binary(heap, bitstring::Binary::from(digest.to_vec())))
}

pub fn make_tuple_2(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let num = match args[0].into_number() {
        Ok(value::Num::Integer(i)) if !i < 0 => i,
        _ => return Err(badarg!()),
    };
    let heap = &process.context_mut().heap;
    let tuple = value::tuple(heap, num as u32);
    for i in 0..num {
        unsafe {
            std::ptr::write(&mut tuple[i as usize], args[1]);
        }
    }
    Ok(Term::from(tuple))
}

pub fn make_tuple_3(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let num = match args[0].into_number() {
        Ok(value::Num::Integer(i)) if !i < 0 => i,
        _ => return Err(badarg!()),
    };
    let heap = &process.context_mut().heap;
    let tuple = value::tuple(heap, num as u32);
    for i in 0..num {
        unsafe {
            std::ptr::write(&mut tuple[i as usize], args[1]);
        }
    }
    let init = Cons::cast_from(&args[2])?;

    for item in init.iter() {
        let t = Tuple::cast_from(&item)?;
        if t.len != 2 {
            return Err(badarg!());
        }
        let n = match t[0].into_number() {
            Ok(value::Num::Integer(i)) if !i < 1 && i - 1 < num => i - 1,
            _ => return Err(badarg!()),
        };
        unsafe {
            std::ptr::write(&mut tuple[n as usize], t[1]);
        }
    }
    Ok(Term::from(tuple))
}

pub fn append_element_2(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let t = Tuple::cast_from(&args[0])?;
    let heap = &process.context_mut().heap;
    let new_tuple = value::tuple(heap, (t.len() + 1) as u32);
    new_tuple[..t.len()].copy_from_slice(&t[..]);
    unsafe {
        std::ptr::write(&mut new_tuple[t.len()], args[1]);
    }
    Ok(Term::from(new_tuple))
}

pub fn setelement_3(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let number = match args[0].into_number() {
        Ok(value::Num::Integer(i)) if !i < 1 => i - 1,
        _ => return Err(badarg!()),
    };
    let t = Tuple::cast_from(&args[1])?;
    if number >= t.len() as i32 {
        return Err(badarg!());
    }
    let heap = &process.context_mut().heap;
    let new_tuple = value::tuple(heap, t.len() as u32);
    unsafe {
        new_tuple[..t.len()].copy_from_slice(&t[..]);
        std::ptr::write(&mut new_tuple[number as usize], args[2]);
    }
    Ok(Term::from(new_tuple))
}

// TODO swap with GetTupleElement ins?
pub fn element_2(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    let number = match args[0].into_number() {
        Ok(value::Num::Integer(i)) if !i < 1 => (i - 1) as usize,
        _ => return Err(badarg!()),
    };
    let t = Tuple::cast_from(&args[1])?;
    if number >= t.len() {
        return Err(badarg!());
    }
    Ok(t[number])
}

pub fn tuple_to_list_1(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let t = Tuple::cast_from(&args[0])?;
    let mut n = t.len();
    let mut list = Term::nil();
    let heap = &process.context_mut().heap;
    while n > 0 {
        n -= 1;
        list = cons!(heap, t[n as usize], list);
    }
    Ok(list)
}

pub fn binary_to_list_1(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let binary = args[0];

    // TODO: extract as macro
    let (bytes, bitoffs, size) = match binary.get_boxed_header() {
        Ok(value::BOXED_BINARY) => {
            let value = &binary.get_boxed_value::<bitstring::RcBinary>().unwrap();
            (&value.data[..], 0, value.data.len())
        }
        Ok(value::BOXED_SUBBINARY) => {
            let value = &binary.get_boxed_value::<bitstring::SubBinary>().unwrap();
            (
                &value.original.data[value.offset..],
                value.bit_offset,
                value.size,
            )
        }
        _ => return Err(badarg!()),
    };

    let res = bitstring::bytes_to_list(
        &process.context_mut().heap,
        Term::nil(),
        bytes,
        size,
        bitoffs,
    );
    Ok(res)
}

/// convert a list of ascii integers to an atom
pub fn list_to_atom_1(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    // Eterm res;
    // byte *buf = (byte *) erts_alloc(ERTS_ALC_T_TMP, MAX_ATOM_SZ_LIMIT);
    // Sint written;
    // int i = erts_unicode_list_to_buf(BIF_ARG_1, buf, MAX_ATOM_CHARACTERS,
    //                                  &written);
    // if (i < 0) {
    // erts_free(ERTS_ALC_T_TMP, (void *) buf);
    // if (i == -2) {
    // BIF_ERROR(BIF_P, SYSTEM_LIMIT);
    // }
    // BIF_ERROR(BIF_P, BADARG);
    // }
    // res = erts_atom_put(buf, written, ERTS_ATOM_ENC_UTF8, 1);
    // ASSERT(is_atom(res));
    // erts_free(ERTS_ALC_T_TMP, (void *) buf);
    // BIF_RET(res);
    let list = Cons::cast_from(&args[0])?;
    let string = value::cons::unicode_list_to_buf(list, atom::MAX_ATOM_CHARS)?;
    let atom = atom::Atom::from(string);
    Ok(Term::atom(atom))
}

/// conditionally convert a list of ascii integers to an atom
pub fn list_to_existing_atom_1(_vm: &Machine, _process: &RcProcess, _args: &[Term]) -> bif::Result {
    // byte *buf = (byte *) erts_alloc(ERTS_ALC_T_TMP, MAX_ATOM_SZ_LIMIT);
    // Sint written;
    // int i = erts_unicode_list_to_buf(BIF_ARG_1, buf, MAX_ATOM_CHARACTERS,
    //                                  &written);
    // if (i < 0) {
    // error:
    // erts_free(ERTS_ALC_T_TMP, (void *) buf);
    // BIF_ERROR(BIF_P, BADARG);
    // } else {
    // Eterm a;

    // if (erts_atom_get((char *) buf, written, &a, ERTS_ATOM_ENC_UTF8)) {
    // erts_free(ERTS_ALC_T_TMP, (void *) buf);
    // BIF_RET(a);
    // } else {
    // goto error;
    // }
    // }
    unimplemented!()
}

// TODO: use Cow
pub fn list_to_iodata(list: Term) -> Result<Vec<u8>, Exception> {
    let mut bytes: Vec<u8> = Vec::new();

    // if nil return an empty string
    if list.is_nil() {
        return Ok(Vec::new());
    }

    if list.is_binary() {
        return Ok(list.to_bytes().unwrap().to_owned());
    }

    let mut stack = Vec::with_capacity(16);
    stack.push(list);

    let mut cons: &Cons;

    while let Some(mut elem) = stack.pop() {
        match elem.into_variant() {
            Variant::Cons(mut ptr) => {
                loop {
                    // tail loop
                    loop {
                        // head loop
                        cons = unsafe { &*ptr };
                        elem = cons.head;

                        match elem.into_variant() {
                            Variant::Integer(i @ 0..=255) => {
                                // append int to bytes
                                bytes.push(i as u8);
                            }
                            Variant::Pointer(..) => match elem.to_bytes() {
                                Some(data) => bytes.extend_from_slice(data),
                                None => return Err(badarg!()),
                            },
                            Variant::Cons(p) => {
                                ptr = p;
                                stack.push(cons.tail);
                                continue; // head loop
                            }
                            Variant::Nil(value::Special::Nil) => break,
                            _ => return Err(badarg!()),
                        }
                        break;
                    }

                    elem = cons.tail;

                    match elem.into_variant() {
                        Variant::Integer(i @ 0..=255) => {
                            // append int to bytes
                            bytes.push(i as u8);
                        }
                        Variant::Pointer(..) => match elem.to_bytes() {
                            Some(data) => bytes.extend_from_slice(data),
                            None => return Err(badarg!()),
                        },
                        Variant::Nil(..) => {}
                        Variant::Cons(p) => {
                            ptr = p;
                            continue; // tail loop
                        }
                        _ => return Err(badarg!()),
                    }
                    break;
                }
            }
            Variant::Pointer(..) => match elem.to_bytes() {
                Some(data) => bytes.extend_from_slice(data),
                None => return Err(badarg!()),
            },
            Variant::Nil(..) => {}
            _ => return Err(badarg!()),
        }
    }

    Ok(bytes)
}

pub fn list_to_binary_1(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let bytes = list_to_iodata(args[0])?;

    let heap = &process.context_mut().heap;
    Ok(Term::binary(heap, bitstring::Binary::from(bytes)))
}
// TODO iolist_to_binary is the same, input can be a binary (is_binary() true), and we just return
// it (badarg on bitstring)
pub fn iolist_to_binary_1(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    if args[0].is_binary() {
        return Ok(args[0]);
    }

    let bytes = list_to_iodata(args[0])?;

    let heap = &process.context_mut().heap;
    Ok(Term::binary(heap, bitstring::Binary::from(bytes)))
}

pub fn iolist_to_iovec_1(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;
    if args[0].is_binary() {
        return Ok(cons!(heap, args[0], Term::nil()));
    }

    // TODO: check for cons or error with badarg

    let mut stack = Vec::with_capacity(16);
    let mut iterator = args[0];
    let mut res = Vec::with_capacity(4);

    loop {
        while let Ok(Cons { head, tail }) = Cons::cast_from(&iterator) {
            match head.into_variant() {
                Variant::Pointer(ptr) => match unsafe { *ptr } {
                    value::BOXED_BINARY | value::BOXED_SUBBINARY => {
                        // append head to result
                        res.push(*head);

                        // TODO: in the future reuse writable binaries as buf
                        iterator = *tail;
                    }
                    _ => return Err(badarg!()),
                },
                Variant::Integer(_i) => {
                    // append byte to buf
                    let mut seq_length = 0;
                    let mut lookahead = iterator;
                    while let Ok(Cons { head, tail: _ }) = Cons::cast_from(&lookahead) {
                        if !head.is_smallint() {
                            break;
                        }
                        seq_length += 1;
                    }
                    let mut buf = Vec::with_capacity(seq_length);

                    while let Ok(Cons { head, tail }) = Cons::cast_from(&iterator) {
                        let i = head.to_int().unwrap();
                        buf.push(i);
                        iterator = *tail;
                        seq_length -= 1;
                        if seq_length == 0 {
                            break;
                        }
                    }
                }
                Variant::Cons(..) | Variant::Nil(..) => {
                    if !tail.is_nil() {
                        stack.push(*tail);
                    }

                    iterator = *head;
                }
                _ => return Err(badarg!()),
            }
            //     } else if (is_small(head)) {
            //         Eterm seq_end;

            //         if (!iol2v_append_byte_seq(state, iterator, &seq_end)) {
            //             goto l_badarg;
            //         }

            //         iterator = seq_end;
        }

        // Handle the list tail

        if iterator.is_binary() {
            // append binary
            res.push(iterator);
        } else if !iterator.is_nil() {
            return Err(badarg!());
        }

        if let Some(item) = stack.pop() {
            iterator = item;
        } else {
            break;
        }
    }

    Ok(res
        .into_iter()
        .rev()
        .fold(Term::nil(), |acc, item| cons!(heap, item, acc)))
}

pub fn unicode_characters_to_binary_2(
    _vm: &Machine,
    process: &RcProcess,
    args: &[Term],
) -> bif::Result {
    let mut bytes: Vec<u8> = Vec::new();
    let heap = &process.context_mut().heap;

    match args[1].into_variant() {
        Variant::Atom(atom::UNICODE) | Variant::Atom(atom::UTF8) | Variant::Atom(atom::LATIN1) => {}
        _ => unimplemented!(), // only unicode atm
    }

    // if nil return an empty string
    if args[0].is_nil() {
        return Ok(Term::binary(heap, bitstring::Binary::new()));
    }

    // if already binary, just return
    if args[0].is_binary() {
        return Ok(args[0]);
    }

    let mut stack = Vec::new();
    let cons = Cons::cast_from(&args[0])?;
    stack.push(cons.iter());

    // TODO fastpath for if [binary]
    while let Some(iter) = stack.last_mut() {
        if let Some(elem) = iter.next() {
            match elem.into_variant() {
                Variant::Integer(i @ 0..=255) => {
                    // append int to bytes
                    bytes.push(i as u8);
                }
                Variant::Cons(ptr) => {
                    // push cons onto stack
                    let cons = unsafe { &*ptr };
                    stack.push(cons.iter())
                }
                Variant::Nil(..) => {}
                Variant::Pointer(..) => match elem.to_bytes() {
                    Some(data) => bytes.extend_from_slice(data),
                    _ => unreachable!("got a weird pointy: {}", elem),
                    // None => return Err(badarg!()),
                },
                _ => unreachable!("got a weird ele: {}", elem),
                // _ => return Err(badarg!()),
            }
        } else {
            stack.pop();
        }
    }

    // TODO: up to here, equivalent to list_to_binary_1
    String::from_utf8(bytes)
        .map(|string| Term::binary(heap, bitstring::Binary::from(string.into_bytes())))
        .map_err(|_| badarg!())
}

pub fn unicode_characters_to_list_2(
    _vm: &Machine,
    process: &RcProcess,
    args: &[Term],
) -> bif::Result {
    let heap = &process.context_mut().heap;

    match args[1].into_variant() {
        Variant::Atom(atom::UNICODE) | Variant::Atom(atom::UTF8) | Variant::Atom(atom::LATIN1) => {}
        _ => unimplemented!(), // only unicode atm
    }

    let bytes = list_to_iodata(args[0])?;

    Ok(bytes.into_iter().rev().fold(Term::nil(), |acc, val| {
        cons!(heap, Term::int(i32::from(val)), acc)
    }))
}

pub fn iolist_size_1(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    // basically list_to_iodata but it counts
    let list = args[0];
    let heap = &process.context_mut().heap;

    // if nil return an empty string
    if list.is_nil() {
        return Ok(Term::int(0));
    }

    // FIXME: to_bytes is potentially inefficient ATM since it can copy (we need to use Cow)

    if list.is_binary() {
        return Ok(Term::uint64(heap, list.to_bytes().unwrap().len() as u64));
    }

    let mut count: usize = 0;

    let mut stack = Vec::new();
    stack.push(list);

    let mut cons: &Cons;

    while let Some(mut elem) = stack.pop() {
        match elem.into_variant() {
            Variant::Cons(mut ptr) => {
                loop {
                    // tail loop
                    loop {
                        // head loop
                        cons = unsafe { &*ptr };
                        elem = cons.head;

                        match elem.into_variant() {
                            Variant::Integer(_i @ 0..=255) => count += 1,
                            Variant::Pointer(..) => match elem.to_bytes() {
                                Some(data) => count += data.len(),
                                None => return Err(badarg!()),
                            },
                            Variant::Cons(p) => {
                                ptr = p;
                                stack.push(cons.tail);
                                continue; // head loop
                            }
                            _ => return Err(badarg!()),
                        }
                        break;
                    }

                    elem = cons.tail;

                    match elem.into_variant() {
                        Variant::Integer(_i @ 0..=255) => count += 1,
                        Variant::Pointer(..) => match elem.to_bytes() {
                            Some(data) => count += data.len(),
                            None => return Err(badarg!()),
                        },
                        Variant::Nil(..) => {}
                        Variant::Cons(p) => {
                            ptr = p;
                            continue; // tail loop
                        }
                        _ => return Err(badarg!()),
                    }
                    break;
                }
            }
            Variant::Pointer(..) => match elem.to_bytes() {
                Some(data) => count += data.len(),
                None => return Err(badarg!()),
            },
            Variant::Nil(..) => {}
            _ => return Err(badarg!()),
        }
    }

    Ok(Term::uint64(heap, count as u64))
}

pub fn binary_to_term_1(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    // TODO: needs to yield mid parsing...
    if let Some(string) = args[0].to_bytes() {
        match Ok(crate::etf::decode(string, &process.context_mut().heap)) {
            Ok(term) => return Ok(term),
            Err::<_, std::io::Error>(error) => panic!("binary_to_term error: {:?}", error),
        };
    }
    Err(badarg!())
}

pub fn term_to_binary_1(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    // TODO: needs to yield mid parsing...
    match crate::etf::encode(args[0]) {
        Ok(term) => Ok(Term::binary(
            &process.context_mut().heap,
            bitstring::Binary::from(term),
        )),
        Err::<_, std::io::Error>(error) => panic!("binary_to_term error: {:?}", error),
    }
}

pub fn term_to_binary_2(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    // TODO: needs to yield mid parsing...
    // TODO: args[1] for compression settings
    match crate::etf::encode(args[0]) {
        Ok(term) => Ok(Term::binary(
            &process.context_mut().heap,
            bitstring::Binary::from(term),
        )),
        Err::<_, std::io::Error>(error) => panic!("binary_to_term error: {:?}", error),
    }
}

pub fn binary_to_atom_2(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    match args[1].into_variant() {
        Variant::Atom(atom::LATIN1) => (),
        Variant::Atom(atom::UNICODE) => (),
        Variant::Atom(atom::UTF8) => (),
        _ => return Err(badarg!()),
    };

    if let Some(bytes) = args[0].to_bytes() {
        return match std::str::from_utf8(bytes) {
            Ok(string) => {
                let atom = atom::Atom::from(string);
                Ok(Term::atom(atom))
            }
            Err(_) => Err(badarg!()),
        };
    }
    Err(badarg!())
}

pub fn atom_to_list_1(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    match args[0].into_variant() {
        Variant::Atom(i) => {
            let string = i.to_str().unwrap();
            let heap = &process.context_mut().heap;

            Ok(bitstring!(heap, string))
        }
        _ => Err(badarg!()),
    }
}

pub fn atom_to_binary_2(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    match args[1].into_variant() {
        Variant::Atom(atom::LATIN1) => (),
        Variant::Atom(atom::UNICODE) => (),
        Variant::Atom(atom::UTF8) => (),
        _ => return Err(badarg!()),
    };

    match args[0].into_variant() {
        Variant::Atom(i) => {
            let string = i.to_str().unwrap();
            let heap = &process.context_mut().heap;

            Ok(Term::binary(
                heap,
                bitstring::Binary::from(string.as_bytes().to_vec()),
            ))
        }
        _ => Err(badarg!()),
    }
}

pub fn pid_to_list_1(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    match args[0].into_variant() {
        Variant::Pid(i) => {
            let string = format!("<0.0.{}>", i); // TODO: proper format
            let heap = &process.context_mut().heap;

            Ok(bitstring!(heap, string))
        }
        _ => Err(badarg!()),
    }
}

pub fn integer_to_list_1(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    match args[0].into_number() {
        Ok(value::Num::Integer(i)) => {
            let string = lexical::to_string(i);
            let heap = &process.context_mut().heap;

            Ok(bitstring!(heap, string))
        }
        Ok(value::Num::Bignum(i)) => {
            let string = i.to_string();
            let heap = &process.context_mut().heap;

            Ok(bitstring!(heap, string))
        }
        _ => {
            println!("integer_to_list_1 called with {}", args[0]);
            Err(badarg!())
        }
    }
}

pub fn integer_to_list_2(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let radix = match args[1].into_variant() {
        Variant::Integer(i) if i >= 2 && i <= 16 => i as u8,
        _ => return Err(badarg!()),
    };

    match args[0].into_number() {
        Ok(value::Num::Integer(i)) => {
            let string = lexical::to_string_radix(i, radix);
            let heap = &process.context_mut().heap;

            Ok(string
                .into_bytes()
                .into_iter()
                .rev()
                .fold(Term::nil(), |acc, val| {
                    cons!(heap, Term::int(i32::from(val)), acc)
                }))
        }
        Ok(value::Num::Bignum(i)) => {
            unimplemented!("integer_to_binary_2 with radix");
            // let string = i.to_bytes_radix(radix);
            // let heap = &process.context_mut().heap;
            // string.into_iter().rev().fold(Term::nil(), |acc, val| cons!(heap, Term::int(int32::from(val), acc)))
        }
        _ => {
            println!("integer_to_list_2 called with {}", args[0]);
            Err(badarg!())
        }
    }
}

pub fn integer_to_binary_1(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    match args[0].into_number() {
        Ok(value::Num::Integer(i)) => {
            let string = lexical::to_string(i).into_bytes();
            let heap = &process.context_mut().heap;

            Ok(Term::binary(heap, bitstring::Binary::from(string)))
        }
        Ok(value::Num::Bignum(bigint)) => {
            let string = bigint.to_str_radix(10).into_bytes();
            let heap = &process.context_mut().heap;

            Ok(Term::binary(heap, bitstring::Binary::from(string)))
        }
        _ => {
            println!("integer_to_list_1 called with {}", args[0]);
            Err(badarg!())
        }
    }
}

pub fn integer_to_binary_2(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let radix = match args[1].into_variant() {
        Variant::Integer(i) if i >= 2 && i <= 16 => i as u8,
        _ => return Err(badarg!()),
    };

    match args[0].into_number() {
        Ok(value::Num::Integer(i)) => {
            let string = lexical::to_string_radix(i, radix).into_bytes();
            let heap = &process.context_mut().heap;

            Ok(Term::binary(heap, bitstring::Binary::from(string)))
        }
        Ok(value::Num::Bignum(bigint)) => {
            let string = bigint.to_str_radix(radix as u32).into_bytes();
            let heap = &process.context_mut().heap;

            Ok(Term::binary(heap, bitstring::Binary::from(string)))
        }
        _ => {
            println!("integer_to_list_2 called with {}", args[0]);
            Err(badarg!())
        }
    }
}

pub fn fun_to_list_1(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    if args[0].get_type() != value::Type::Closure {
        return Err(badarg!());
    }

    let string = format!("{}", args[0]);
    let heap = &process.context_mut().heap;

    Ok(bitstring!(heap, string))
}

pub fn ref_to_list_1(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    if args[0].get_type() != value::Type::Ref {
        return Err(badarg!());
    }

    let string = format!("{}", args[0]);
    let heap = &process.context_mut().heap;

    Ok(bitstring!(heap, string))
}

pub fn binary_to_integer_1(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    // list to string
    let bytes = match args[0].to_bytes() {
        Some(b) => b,
        _ => return Err(badarg!()),
    };
    match lexical::parse::<i32, _>(bytes) {
        Ok(i) => Ok(Term::int(i)),
        Err(err) => panic!("errored with {:?}", err), //TODO bigint
    }
}

pub fn list_to_integer_1(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    // list to string
    let cons = Cons::cast_from(&args[0])?;
    let string = value::cons::unicode_list_to_buf(cons, 2048)?;
    match lexical::parse::<i32, _>(string) {
        Ok(i) => Ok(Term::int(i)),
        Err(err) => panic!("errored with {:?}", err), //TODO
    }
}

pub fn string_list_to_integer_1(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    // list to string
    let heap = &process.context_mut().heap;

    // either binary or charlist
    let string = if args[0].is_binary() {
        args[0].to_bytes().unwrap().to_owned()
    } else {
        let cons = Cons::cast_from(&args[0])?;
        value::cons::unicode_list_to_buf(cons, 2048)?
            .as_bytes()
            .to_owned()
    };

    match lexical::parse::<i32, _>(string.clone()) {
        // the api is not great here, need to clone
        Ok(i) => Ok(tup2!(heap, Term::int(i), Term::nil())),
        Err(err) => {
            match err {
                lexical::Error {
                    code: lexical::ErrorCode::InvalidDigit,
                    index: 0,
                } => Ok(tup2!(heap, atom!(ERROR), atom!(NO_INTEGER))),
                lexical::Error {
                    code: lexical::ErrorCode::InvalidDigit,
                    index: n,
                } => {
                    // TODO: tests
                    Ok(tup2!(
                        heap,
                        Term::int(lexical::parse::<i32, _>(&string[..n]).unwrap()),
                        string[n..].iter().rev().fold(Term::nil(), |acc, b| cons!(
                            heap,
                            Term::int(i32::from(*b)),
                            acc
                        ))
                    ))
                }
                err => panic!("errored with {:?}", err), //TODO
            }
        }
    }
}

pub fn list_to_float_1(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    // list to string
    let cons = Cons::cast_from(&args[0])?;
    let string = value::cons::unicode_list_to_buf(cons, 2048)?;
    match lexical::parse::<f64, _>(string) {
        Ok(f) => Ok(Term::from(f)),
        Err(err) => panic!("errored with {:?}", err), //TODO
    }
}

pub fn list_to_tuple_1(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    // list to tuple
    let heap = &process.context_mut().heap;

    let mut tmp = args[0];
    let mut arity = 0;

    while let Ok(value::Cons { tail, .. }) = tmp.cast_into() {
        arity += 1;
        tmp = *tail
    }

    if !tmp.is_nil() {
        // Must be well-formed list
        return Err(badarg!());
    }

    // allocate tuple, traverse the list and assign to tup
    let tuple = value::tuple(heap, arity);
    let mut i = 0;
    let mut tmp = args[0];

    while let Ok(value::Cons { head, tail }) = tmp.cast_into() {
        unsafe {
            std::ptr::write(&mut tuple[i], *head);
        }
        i += 1;
        tmp = *tail
    }
    Ok(Term::from(tuple))
}

pub fn display_1(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    println!("{}\r", args[0]);
    Ok(atom!(TRUE))
}

pub fn display_string_1(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    let cons = Cons::cast_from(&args[0])?;
    let string = value::cons::unicode_list_to_buf(cons, 2048)?;
    print!("{}", string);
    Ok(atom!(TRUE))
}

pub fn display_nl_0(_vm: &Machine, _process: &RcProcess, _args: &[Term]) -> bif::Result {
    println!("\r");
    Ok(atom!(TRUE))
}

/// erlang:'++'/2
///
/// Adds a list to another (LHS ++ RHS). For historical reasons this is implemented by copying LHS
/// and setting its tail to RHS without checking that RHS is a proper list. [] ++ 'not_a_list' will
/// therefore result in 'not_a_list', and [1,2] ++ 3 will result in [1,2|3], and this is a bug that
/// we have to live with.
pub fn append_2(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let lhs = args[0];
    let rhs = args[1];

    let heap = &process.context_mut().heap;

    // This is buggy but expected, `[] ++ 'not_a_list'` has always resulted in 'not_a_list'.
    if lhs.is_nil() {
        return Ok(rhs);
    }

    if let Ok(value::Cons { head, tail }) = lhs.cast_into() {
        // keep copying lhs until we reach the tail, point it to rhs
        let mut iter = tail;

        let c = heap.alloc(value::Cons {
            head: *head,
            tail: Term::nil(),
        });

        let mut ptr = c as *mut value::Cons;

        while let Ok(value::Cons { head, tail }) = iter.cast_into() {
            let new_cons = heap.alloc(value::Cons {
                head: *head,
                tail: Term::nil(),
            });

            let prev = unsafe { &mut (*ptr).tail };
            ptr = new_cons as *mut value::Cons;
            std::mem::replace(prev, Term::from(new_cons));

            iter = tail;
        }

        if !iter.is_nil() {
            // tail was a badly formed list
            return Err(badarg!());
        }

        // now link the copy to the rhs
        unsafe {
            (*ptr).tail = rhs;
        }
        return Ok(Term::from(c));
    }

    // assert!(!(BIF_P->flags & F_DISABLE_GC));
    Err(badarg!())
}

pub fn subtract_2(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    // TODO: a more efficient impl
    // calculate len A
    // calculate len B
    // copy B to vec
    // subtract B from A, shrinking B as els consumed
    // copy result to proc heap
    //
    // use linear scan on small elems
    // use hashset for larger one

    if args[0].is_nil() || args[1].is_nil() {
        return Ok(args[0]);
    }

    let mut a: Vec<Term> = Cons::cast_from(&args[0])?.into_iter().copied().collect();
    let b = Cons::cast_from(&args[1])?;

    for item in b {
        a.iter().position(|x| *x == *item).map(|i| a.remove(i));
    }

    let heap = &process.context_mut().heap;
    Ok(Cons::from_iter(a.into_iter(), heap))
}

pub fn make_ref_0(vm: &Machine, process: &RcProcess, _args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;
    let reference = vm.next_ref();

    // TODO: heap allocating these is not ideal
    Ok(Term::reference(heap, reference))
}

// for the time being, these two functions are constant since we don't do distributed
pub fn node_0(_vm: &Machine, _process: &RcProcess, _args: &[Term]) -> bif::Result {
    Ok(atom!(NO_NODE_NO_HOST))
}
pub fn node_1(_vm: &Machine, _process: &RcProcess, _args: &[Term]) -> bif::Result {
    Ok(atom!(NO_NODE_NO_HOST))
}

pub fn processes_0(vm: &Machine, process: &RcProcess, _args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;

    let pids = {
        let process_table = vm.process_table.lock();
        process_table.all()
    };

    let res = pids
        .into_iter()
        .rev()
        .fold(Term::nil(), |acc, pid| cons!(heap, Term::pid(pid), acc));
    Ok(res)
}

pub fn and_2(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    match (args[0].to_bool(), args[1].to_bool()) {
        (Some(true), Some(true)) => Ok(atom!(TRUE)),
        (Some(_), Some(_)) => Ok(atom!(FALSE)),
        _ => Err(badarg!()),
    }
}

pub fn or_2(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    match (args[0].to_bool(), args[1].to_bool()) {
        (Some(false), Some(false)) => Ok(atom!(FALSE)),
        (Some(_), Some(_)) => Ok(atom!(TRUE)),
        _ => Err(badarg!()),
    }
}

pub fn xor_2(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    match (args[0].to_bool(), args[1].to_bool()) {
        (Some(true), Some(false)) => Ok(atom!(TRUE)),
        (Some(false), Some(true)) => Ok(atom!(TRUE)),
        (Some(_), Some(_)) => Ok(atom!(FALSE)),
        _ => Err(badarg!()),
    }
}

pub fn not_1(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    match args[0].to_bool() {
        Some(true) => Ok(atom!(FALSE)),
        Some(false) => Ok(atom!(TRUE)),
        None => Err(badarg!()),
    }
}

pub fn sgt_2(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    Ok(Term::boolean(
        args[0].cmp(&args[1]) == std::cmp::Ordering::Greater,
    ))
}

pub fn sge_2(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    // greater or equal
    Ok(Term::boolean(
        args[0].cmp(&args[1]) != std::cmp::Ordering::Less,
    ))
}

pub fn slt_2(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    Ok(Term::boolean(
        args[0].cmp(&args[1]) == std::cmp::Ordering::Less,
    ))
}

pub fn sle_2(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    // less or equal
    Ok(Term::boolean(
        args[0].cmp(&args[1]) != std::cmp::Ordering::Greater,
    ))
}

pub fn seq_2(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    Ok(Term::boolean(args[0].eq(&args[1])))
}

pub fn seqeq_2(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    Ok(Term::boolean(
        args[0].cmp(&args[1]) == std::cmp::Ordering::Equal,
    ))
}

pub fn sneq_2(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    Ok(Term::boolean(!args[0].eq(&args[1])))
}

pub fn sneqeq_2(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    Ok(Term::boolean(
        args[0].cmp(&args[1]) != std::cmp::Ordering::Equal,
    ))
}

pub fn bor_2(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    let i1 = match args[0].into_variant() {
        Variant::Integer(i) => i,
        _ => return Err(badarg!()),
    };
    let i2 = match args[1].into_variant() {
        Variant::Integer(i) => i,
        _ => return Err(badarg!()),
    };
    Ok(Term::int(i1 | i2))
}

pub fn band_2(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    let i1 = match args[0].into_variant() {
        Variant::Integer(i) => i,
        _ => return Err(badarg!()),
    };
    let i2 = match args[1].into_variant() {
        Variant::Integer(i) => i,
        _ => return Err(badarg!()),
    };
    Ok(Term::int(i1 & i2))
}

pub fn bxor_2(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    let i1 = match args[0].into_variant() {
        Variant::Integer(i) => i,
        _ => return Err(badarg!()),
    };
    let i2 = match args[1].into_variant() {
        Variant::Integer(i) => i,
        _ => return Err(badarg!()),
    };
    Ok(Term::int(i1 ^ i2))
}

pub fn bsl_2(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    let i1 = match args[0].into_variant() {
        Variant::Integer(i) => i,
        _ => return Err(badarg!()),
    };
    let i2 = match args[1].into_variant() {
        Variant::Integer(i) => i,
        _ => return Err(badarg!()),
    };
    // TODO: need to use overflowing_shl
    Ok(Term::int(i1 << i2))
}

pub fn bsr_2(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    let i1 = match args[0].into_variant() {
        Variant::Integer(i) => i,
        _ => return Err(badarg!()),
    };
    let i2 = match args[1].into_variant() {
        Variant::Integer(i) => i,
        _ => return Err(badarg!()),
    };
    // TODO: need to use overflowing_shr
    Ok(Term::int(i1 >> i2))
}

pub fn bnot_1(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    let i1 = match args[0].into_variant() {
        Variant::Integer(i) => i,
        _ => return Err(badarg!()),
    };
    Ok(Term::int(!i1))
}

pub fn sminus_1(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    // if number, negate number else return error badarith
    let heap = &process.context_mut().heap;
    match args[0].into_number() {
        Ok(value::Num::Integer(i)) => Ok(Term::int(-i)),
        Ok(value::Num::Float(i)) => Ok(Term::from(-i)),
        Ok(value::Num::Bignum(i)) => Ok(Term::bigint(heap, -i)),
        _ => Err(Exception::new(Reason::EXC_BADARITH)),
    }
}

pub fn splus_1(_vm: &Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    // if number, return number else return error badarith
    match args[0].into_number() {
        Ok(_) => Ok(args[0]),
        _ => Err(Exception::new(Reason::EXC_BADARITH)),
    }
}

pub fn make_fun_3(_vm: &Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    // module, func, arity? return BOXED_EXPORT
    let heap = &process.context_mut().heap;
    match (
        args[0].into_variant(),
        args[1].into_variant(),
        args[2].into_variant(),
    ) {
        (Variant::Atom(m), Variant::Atom(f), Variant::Integer(a)) if a > 0 => {
            Ok(Term::export(heap, crate::module::MFA(m, f, a as u32)))
        }
        _ => Err(badarg!()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::module;
    use crate::process;
    use crate::value::Cons;

    macro_rules! str_to_atom {
        ($str:expr) => {
            Term::atom(crate::atom::Atom::from($str))
        };
    }

    #[test]
    fn test_make_tuple_2() {
        let vm = Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm, 0, 0, module).unwrap();
        let number = Term::int(2);
        let default = str_to_atom!("test");
        let args = vec![number, default];

        let res = make_tuple_2(&vm, &process, &args);
        let x = res.unwrap();
        assert!(x.is_tuple());
        if let Ok(tuple) = x.cast_into() {
            let tuple: &Tuple = tuple;
            tuple
                .iter()
                .for_each(|x| assert_eq!(x, &str_to_atom!("test")));
            assert_eq!(tuple.len, 2);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_make_tuple_2_bad_arg_wrong_type_of_tuple() {
        let vm = Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm, 0, 0, module).unwrap();
        let number = Term::from(2.1);
        let default = str_to_atom!("test");
        let args = vec![number, default];

        let res = make_tuple_2(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_make_tuple_2_bad_arg_negative_number() {
        let vm = Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm, 0, 0, module).unwrap();
        let number = Term::int(-1);
        let default = str_to_atom!("test");
        let args = vec![number, default];

        let res = make_tuple_2(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }
    #[test]
    fn test_tuple_make_tuple_3() {
        let vm = Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm, 0, 0, module).unwrap();
        let heap = &process.context_mut().heap;

        let number = Term::int(5);
        let default = Term::from(1);
        let init_list = Cons::from_iter(
            vec![
                tup2!(&heap, Term::int(2), str_to_atom!("ignored")),
                tup2!(&heap, Term::int(5), str_to_atom!("zz")),
                tup2!(&heap, Term::int(2), str_to_atom!("aa")),
            ]
            .into_iter(),
            &heap,
        );

        let args = vec![number, default, init_list];

        let res = make_tuple_3(&vm, &process, &args);
        let x = res.unwrap();
        assert!(x.is_tuple());
        if let Ok(tuple) = x.cast_into() {
            let tuple: &Tuple = tuple;
            assert_eq!(tuple.len, 5);
            assert_eq!(tuple[0], Term::from(1));
            assert_eq!(tuple[1], str_to_atom!("aa"));
            assert_eq!(tuple[2], Term::from(1));
            assert_eq!(tuple[3], Term::from(1));
            assert_eq!(tuple[4], str_to_atom!("zz"));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_tuple_make_tuple_3_bad_arg_wrong_type_of_number() {
        let vm = Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm, 0, 0, module).unwrap();
        let heap = &process.context_mut().heap;

        let number = Term::from(2.1);
        let default = Term::from(1);
        let init_list = iter_to_list!(
            &heap,
            vec![
                tup2!(&heap, Term::int(2), str_to_atom!("ignored")),
                tup2!(&heap, Term::int(4), str_to_atom!("zz")),
                tup2!(&heap, Term::int(2), str_to_atom!("aa")),
            ]
            .iter()
            .map(|x| *x)
        );

        let args = vec![number, default, init_list];

        let res = make_tuple_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_tuple_make_tuple_3_bad_arg_negative_number() {
        let vm = Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm, 0, 0, module).unwrap();
        let heap = &process.context_mut().heap;

        let number = Term::int(-1);
        let default = Term::from(1);
        let init_list = iter_to_list!(
            &heap,
            vec![
                tup2!(&heap, Term::int(2), str_to_atom!("ignored")),
                tup2!(&heap, Term::int(4), str_to_atom!("zz")),
                tup2!(&heap, Term::int(2), str_to_atom!("aa")),
            ]
            .iter()
            .map(|x| *x)
        );

        let args = vec![number, default, init_list];

        let res = make_tuple_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_tuple_make_tuple_3_bad_arg_wrong_type_of_init_list() {
        let vm = Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm, 0, 0, module).unwrap();
        let _heap = &process.context_mut().heap;

        let number = Term::int(5);
        let default = Term::from(1);
        let init_list = Term::from(1);

        let args = vec![number, default, init_list];

        let res = make_tuple_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_tuple_make_tuple_3_bad_arg_wrong_structure_init_list() {
        let vm = Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm, 0, 0, module).unwrap();
        let heap = &process.context_mut().heap;

        let number = Term::int(5);
        let default = Term::from(1);
        let init_list = iter_to_list!(
            &heap,
            vec![
                tup2!(&heap, Term::int(2), str_to_atom!("ignored")),
                tup3!(&heap, Term::int(4), str_to_atom!("zz"), Term::from(1)),
                tup2!(&heap, Term::from(2.0), str_to_atom!("aa")),
            ]
            .iter()
            .map(|x| *x)
        );

        let args = vec![number, default, init_list];

        let res = make_tuple_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_tuple_make_tuple_3_bad_arg_init_list_out_of_range() {
        let vm = Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm, 0, 0, module).unwrap();
        let heap = &process.context_mut().heap;

        let number = Term::int(5);
        let default = Term::from(1);
        let init_list = iter_to_list!(
            &heap,
            vec![
                tup2!(&heap, Term::int(2), str_to_atom!("ignored")),
                tup2!(&heap, Term::int(4), str_to_atom!("zz")),
                tup2!(&heap, Term::int(10), str_to_atom!("aa")),
            ]
            .iter()
            .map(|x| *x)
        );

        let args = vec![number, default, init_list];

        let res = make_tuple_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_append_element_2() {
        let vm = Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm, 0, 0, module).unwrap();
        let heap = &process.context_mut().heap;
        let tuple = tup2!(&heap, Term::int(0), Term::int(1));
        let append = Term::int(2);
        let args = vec![tuple, append];

        let res = append_element_2(&vm, &process, &args);
        let x = res.unwrap();
        assert!(x.is_tuple());
        if let Ok(tuple) = x.cast_into() {
            let tuple: &Tuple = tuple;
            assert_eq!(tuple.len, 3);
            for (i, x) in tuple.iter().enumerate() {
                assert_eq!(x, &Term::int(i as i32));
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_append_element_2_bad_arg_wrong_type_of_tuple() {
        let vm = Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm, 0, 0, module).unwrap();
        let bad_tuple = Term::int(0);
        let append = Term::int(2);
        let args = vec![bad_tuple, append];

        let res = append_element_2(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_setelement_3() {
        let vm = Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm, 0, 0, module).unwrap();
        let heap = &process.context_mut().heap;
        let index = Term::int(2);
        let tuple = tup3!(&heap, str_to_atom!("test"), Term::from(1), Term::from(2));
        let value = Term::from(99);
        let args = vec![index, tuple, value];

        let res = setelement_3(&vm, &process, &args);
        let x = res.unwrap();
        assert!(x.is_tuple());
        if let Ok(tuple) = x.cast_into() {
            let tuple: &Tuple = tuple;
            assert_eq!(tuple.len, 3);
            assert_eq!(tuple[0], str_to_atom!("test"));
            assert_eq!(tuple[1], Term::from(99));
            assert_eq!(tuple[2], Term::from(2));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_setelement_3_bad_arg_wrong_type_of_index() {
        let vm = Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm, 0, 0, module).unwrap();
        let heap = &process.context_mut().heap;
        let index = Term::from(1.1);
        let tuple = tup3!(&heap, str_to_atom!("test"), Term::from(1), Term::from(2));
        let value = Term::from(99);
        let args = vec![index, tuple, value];

        let res = setelement_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_setelement_3_bad_arg_wrong_type_of_tuple() {
        let vm = Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm, 0, 0, module).unwrap();
        let _heap = &process.context_mut().heap;
        let index = Term::int(1);
        let tuple = Term::from(1);
        let value = Term::from(99);
        let args = vec![index, tuple, value];

        let res = setelement_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_setelement_3_bad_arg_tuple_out_of_range() {
        let vm = Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm, 0, 0, module).unwrap();
        let heap = &process.context_mut().heap;
        let index = Term::int(4);
        let tuple = tup3!(&heap, str_to_atom!("test"), Term::from(1), Term::from(2));
        let value = Term::from(99);
        let args = vec![index, tuple, value];

        let res = setelement_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_tuple_to_list_1() {
        let vm = Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm, 0, 0, module).unwrap();
        let heap = &process.context_mut().heap;
        let tuple = tup2!(
            &heap,
            str_to_atom!("test"),
            tup2!(&heap, Term::from(1), Term::from(2))
        );
        let args = vec![tuple];

        let res = tuple_to_list_1(&vm, &process, &args);
        let x = res.unwrap();
        assert!(x.is_list());
        if let Ok(cons) = x.cast_into() {
            let cons: &value::Cons = cons;
            assert_eq!(cons.iter().count(), 2);
            let mut iter = cons.iter();
            assert_eq!(iter.next().unwrap(), &str_to_atom!("test"));
            assert_eq!(
                iter.next().unwrap(),
                &tup2!(&heap, Term::from(1), Term::from(2))
            );
        } else {
            panic!();
        }
    }

    #[test]
    fn test_tuple_to_list_1_bad_arg_wrong_type_of_tuple() {
        let vm = Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm, 0, 0, module).unwrap();
        let bad_tuple = Term::from(1);
        let args = vec![bad_tuple];

        let res = tuple_to_list_1(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_list_to_iodata() {
        let vm = Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm, 0, 0, module).unwrap();
        let heap = &process.context_mut().heap;

        let binary = crate::bitstring::Binary::from(vec![0xAB, 0xCD, 0xEF]);
        let binary = Term::binary(heap, binary);

        let list = cons!(
            heap,
            cons!(
                heap,
                cons!(heap, Term::int(1), cons!(heap, Term::int(2), Term::nil())),
                cons!(heap, Term::int(3), Term::nil())
            ),
            binary
        );
        // [[1, 2], 3 | <<0xAB, 0xCD, 0xEF>>]

        let res = list_to_iodata(list);
        assert_eq!(Ok(vec![1, 2, 3, 0xAB, 0xCD, 0xEF]), res)
    }
}
