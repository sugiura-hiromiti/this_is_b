#![cfg_attr(not(feature = "std"), no_std)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use core::{
	convert::Infallible,
	fmt::Debug,
	ops::{ControlFlow, FromResidual, Residual, Try},
};

/// b stands for boolish, branch and binary
pub enum B<S, T,>
{
	X(S,),
	Y(T,),
}

impl<S, T,> B<S, T,>
{
	pub fn is_x(&self,) -> bool
	{
		matches!(self, B::X(_),)
	}

	pub fn is_y(&self,) -> bool
	{
		!self.is_x()
	}

	pub fn map<X,>(self, f: impl FnOnce(S,) -> X,) -> B<X, T,>
	{
		match self {
			Self::X(s,) => B::X(f(s,),),
			Self::Y(t,) => B::Y(t,),
		}
	}
}

impl<S, T, T2,> FromResidual<B<Infallible, T2,>,> for B<S, T,>
where T: From<T2,>
{
	#[track_caller]
	fn from_residual(residual: B<Infallible, T2,>,) -> Self
	{
		match residual {
			B::X(_i,) => unreachable!(),
			B::Y(t,) => Self::Y(t.into(),),
		}
	}
}

impl<S, T: From<E,>, E: core::error::Error,>
	FromResidual<Result<Infallible, E,>,> for B<S, T,>
{
	#[track_caller]
	fn from_residual(residual: Result<Infallible, E,>,) -> Self
	{
		match residual {
			Ok(_i,) => unreachable!(),
			Err(e,) => Self::Y(T::from(e,),),
		}
	}
}

impl<S, T,> Try for B<S, T,>
{
	type Output = S;
	type Residual = B<Infallible, T,>;

	fn from_output(output: Self::Output,) -> Self
	{
		Self::X(output,)
	}

	fn branch(self,) -> core::ops::ControlFlow<Self::Residual, Self::Output,>
	{
		match self {
			Self::X(s,) => ControlFlow::Continue(s,),
			Self::Y(t,) => ControlFlow::Break(B::Y(t,),),
		}
	}
}

impl<S, T,> Residual<S,> for B<Infallible, T,>
{
	type TryType = B<S, T,>;
}

#[cfg(feature = "std")]
impl<S, T: std::fmt::Display,> std::process::Termination for B<S, T,>
{
	fn report(self,) -> std::process::ExitCode
	{
		match self {
			Self::X(_,) => std::process::ExitCode::SUCCESS,
			Self::Y(t,) => {
				eprintln!("{t:#}");
				std::process::ExitCode::FAILURE
			},
		}
	}
}

pub trait ReShape<O, C,>
{
	fn reshape(self, ctx: C,) -> O;
}

impl<T, E,> ReShape<B<T, E,>, (),> for Result<T, E,>
{
	fn reshape(self, _ctx: (),) -> B<T, E,>
	{
		match self {
			Self::Ok(t,) => B::X(t,),
			Self::Err(e,) => B::Y(e,),
		}
	}
}

impl<T, E: From<C,>, C,> ReShape<B<T, E,>, C,> for Option<T,>
{
	fn reshape(self, ctx: C,) -> B<T, E,>
	{
		match self {
			Self::Some(t,) => B::X(t,),
			Self::None => B::Y(E::from(ctx,),),
		}
	}
}

impl<T, E,> ReShape<Result<T, E,>, (),> for B<T, E,>
{
	fn reshape(self, _ctx: (),) -> Result<T, E,>
	{
		match self {
			Self::X(t,) => Ok(t,),
			Self::Y(e,) => Err(e,),
		}
	}
}
impl<T, E,> ReShape<Option<T,>, (),> for B<T, E,>
{
	fn reshape(self, _ctx: (),) -> Option<T,>
	{
		match self {
			Self::X(t,) => Some(t,),
			Self::Y(_,) => None,
		}
	}
}

pub trait Container
{
	type T;
	type E;
	fn unwrap(self,) -> Self::T;
	fn expect(self, msg: &str,) -> Self::T;
	fn unwrap_inv(self,) -> Self::E;
	fn expect_inv(self, msg: &str,) -> Self::E;
}

impl<T, E: Debug,> Container for B<T, E,>
{
	type E = E;
	type T = T;

	fn unwrap(self,) -> Self::T
	{
		let a: Result<_, _,> = self.reshape((),);
		a.unwrap()
	}

	fn expect(self, msg: &str,) -> Self::T
	{
		let a: Result<_, _,> = self.reshape((),);
		a.expect(msg,)
	}

	fn unwrap_inv(self,) -> Self::E
	{
		match self {
			Self::X(_,) => panic!(),
			Self::Y(s,) => s,
		}
	}

	fn expect_inv(self, msg: &str,) -> Self::E
	{
		match self {
			Self::X(_,) => panic!("{msg}"),
			Self::Y(s,) => s,
		}
	}
}
