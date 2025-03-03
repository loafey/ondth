use faststr::FastStr;

use crate::tokenizer::{Symbol, Token};
use std::{
    collections::HashMap,
    io::{self, Result},
    iter::Peekable,
    ops::Div,
    vec::IntoIter,
};

#[macro_use]
mod macros {
    macro_rules! get_token {
        ($context:literal,$token_itr:ident, $token:pat) => {{
            let token = $token_itr.next();
            match token {
                Some(t @ $token) => t,
                Some(token) => error_token($context, token)?,
                None => Err(error_eof())?,
            }
        }};
    }
}

/// Helper struct for when trying the parser can return multiple things.
pub enum Either<L, R> {
    /// The left value
    Left(L),
    /// The right value
    Right(R),
}

/// Alias for a collection of [Planes][Plane] which define a brush.
pub type Brush = Vec<Plane>;

/// The definition of a plane.
/// Planes are defined using the [parametric equation](https://mathworld.wolfram.com/Plane.html).
/// Also contains a texture x and y offset, rotation, x and y scale, and its path.
#[derive(Debug, Clone, PartialEq)]
pub struct Plane {
    /// The first vector defining a plane
    pub p1: Vector,
    /// The second vector defining a plane
    pub p2: Vector,
    /// The third vector defining a plane
    pub p3: Vector,
    /// The path to the planes texture
    pub texture: FastStr,
    /// The x offset of the planes texture
    pub x_offset: TextureOffset,
    /// The y offset of the planes texture
    pub y_offset: TextureOffset,
    /// The radian rotation of the planes texture
    pub rotation: f32,
    /// The x scale of the planes texture
    pub x_scale: f32,
    /// The y scale of the planes texture
    pub y_scale: f32,
}

/// The texture offset.
/// Can either be in the original format ([TextureOffset::Simple]), or Valve's
/// V220 format.
#[derive(Clone, Copy, PartialEq)]
pub enum TextureOffset {
    /// A simple texture offset
    Simple(f32),
    /// A texture offset in the V220 format.
    /// The first three floats is the texture vector.
    V220(f32, f32, f32, f32),
}
impl std::fmt::Debug for TextureOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Simple(arg0) => write!(f, "Simple[{arg0}]"),
            Self::V220(arg0, arg1, arg2, arg3) => write!(f, "V200[{arg0}, {arg1}, {arg2}, {arg3}]"),
        }
    }
}

/// A simple 3D vector.
#[derive(Clone, Copy, PartialEq)]
pub struct Vector(pub f32, pub f32, pub f32);
impl Div<f32> for Vector {
    type Output = Vector;

    fn div(self, rhs: f32) -> Self::Output {
        Self(self.0 / rhs, self.1 / rhs, self.2 / rhs)
    }
}
impl std::fmt::Debug for Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vec[{}, {}, {}]", self.0, self.1, self.2)
    }
}

#[derive(Debug, Clone)]
struct Attribute(FastStr, FastStr);

/// A map entity. Consists of [brushes][Brush] and attributes for
/// defining behavior. The attribute `classname` defines the
/// type of entity.
#[derive(Debug, Default)]
pub struct Entity {
    /// The set of entity attributes.
    pub attributes: HashMap<FastStr, FastStr>,
    /// The planes which define the shape.
    /// Might be empty, if it is for example an enemy.
    pub brushes: Vec<Brush>,
}

type TokenItr<'a> = &'a mut Peekable<IntoIter<Token>>;

/// Parses a list of tokens into a usable [Vec] of [entities][Entity].
///
/// # Errors
/// Fails if there are invalid tokens, i.e out of order etc.
pub fn parser(tokens: Vec<Token>) -> Result<Vec<Entity>> {
    let mut token_itr = tokens.into_iter().peekable();
    map_items(&mut token_itr)
}

fn map_items(toks: TokenItr<'_>) -> Result<Vec<Entity>> {
    let mut res = Vec::new();
    while let Some(ent) = map_entity(toks)? {
        res.push(ent);
    }
    Ok(res)
}

fn entity_data(toks: TokenItr<'_>) -> Result<Option<Either<Attribute, Brush>>> {
    match toks.next().ok_or(error_eof())? {
        Token(Symbol::String(lhs), ..) => Ok(Some(Either::Left(entity_attribute(toks, lhs)?))),
        Token(Symbol::LBrack, ..) => Ok(Some(Either::Right(brush(toks)?))),
        Token(Symbol::RBrack, ..) => Ok(None),
        token => error_token("entity content", token),
    }
}

fn brush(toks: TokenItr<'_>) -> Result<Brush> {
    let mut brush = Vec::new();

    while let Some(plane) = plane(toks)? {
        brush.push(plane);
    }

    Ok(brush)
}

fn vector(toks: TokenItr<'_>) -> Result<Vector> {
    get_token!("vector start", toks, Token(Symbol::LParan, ..));

    let x = float32(toks)?;
    let y = float32(toks)?;
    let z = float32(toks)?;

    get_token!("vector end", toks, Token(Symbol::RParan, ..));

    Ok(Vector(x, y, z))
}

fn float32(toks: TokenItr<'_>) -> Result<f32> {
    let Token(Symbol::Number(y), col, row) =
        get_token!("float", toks, Token(Symbol::Number(..), ..))
    else {
        unreachable!()
    };
    y.parse().io_error(col, row)
}

fn texture_offset(toks: TokenItr<'_>) -> Result<TextureOffset> {
    match toks.next() {
        Some(Token(Symbol::LSquare, ..)) => {
            let x = float32(toks)?;
            let y = float32(toks)?;
            let z = float32(toks)?;
            let w = float32(toks)?;

            get_token!("texture offset", toks, Token(Symbol::RSquare, ..));

            Ok(TextureOffset::V220(x, y, z, w))
        }
        Some(Token(Symbol::Number(x), col, row)) => {
            let x = x.parse().io_error(col, row)?;
            Ok(TextureOffset::Simple(x))
        }
        Some(token) => error_token("texture offset", token)?,
        None => Err(error_eof())?,
    }
}

fn plane(toks: TokenItr<'_>) -> Result<Option<Plane>> {
    if let Some(Token(Symbol::RBrack, ..)) = toks.peek() {
        toks.next();
        return Ok(None);
    }

    let x = vector(toks)?;
    let y = vector(toks)?;
    let z = vector(toks)?;

    let Token(Symbol::Texture(texture), ..) =
        get_token!("plane texture", toks, Token(Symbol::Texture(..), ..))
    else {
        unreachable!()
    };
    let texture = FastStr::from(texture);

    let x_offset = texture_offset(toks)?;
    let y_offset = texture_offset(toks)?;

    let rotation = float32(toks)?;
    let x_scale = float32(toks)?;
    let y_scale = float32(toks)?;

    let plane = Plane {
        p1: x,
        p2: y,
        p3: z,
        texture,
        x_offset,
        y_offset,
        rotation,
        x_scale,
        y_scale,
    };

    Ok(Some(plane))
}

fn entity_attribute(toks: TokenItr<'_>, lhs: String) -> Result<Attribute> {
    match toks.next().ok_or(error_eof())? {
        Token(Symbol::String(rhs), ..) => Ok(Attribute(
            FastStr::from(lhs[1..lhs.len() - 1].to_string()),
            FastStr::from(rhs[1..rhs.len() - 1].to_string()),
        )),
        token => error_token("entity attribute", token),
    }
}

fn map_entity(toks: TokenItr<'_>) -> Result<Option<Entity>> {
    match toks.next() {
        Some(token) => match token.0 {
            Symbol::LBrack => {
                let mut entity = Entity::default();
                while let Some(data) = entity_data(toks)? {
                    match data {
                        Either::Left(Attribute(lhs, rhs)) => {
                            entity.attributes.insert(lhs, rhs);
                        }
                        Either::Right(planes) => entity.brushes.push(planes),
                    }
                }
                Ok(Some(entity))
            }
            Symbol::RBrack => Ok(None),
            _ => error_token("map_entity", token),
        },
        None => Ok(None),
    }
}

fn error_token<T>(parsing_type: &str, token: Token) -> Result<T> {
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "unexpected token at {}:{} when parsing \"{parsing_type}\": \"{:?}\" ",
            token.2, token.1, token.0
        ),
    ))
}

fn error_eof() -> io::Error {
    io::Error::new(io::ErrorKind::UnexpectedEof, "sudden EOF")
}

trait ToIOError<T> {
    fn io_error(self, col: usize, row: usize) -> Result<T>;
}
impl<T> ToIOError<T> for std::result::Result<T, std::num::ParseFloatError> {
    fn io_error(self, col: usize, row: usize) -> Result<T> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{e} at {row}:{col}"),
            )),
        }
    }
}
