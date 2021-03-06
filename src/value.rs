use crate::symbol::Symbol;
use crate::type_tag::TypeTag;
use crate::environment::Environment;
use crate::ifn::IFn;
use crate::persistent_list::{ToPersistentList,ToPersistentListIter,PersistentList};
use crate::persistent_list::PersistentList::{Empty,Cons};
use crate::persistent_vector::{ToPersistentVector,ToPersistentVectorIter,PersistentVector};
use crate::persistent_list_map::{PersistentListMap,ToPersistentListMapIter};
use crate::lambda;
use crate::maps::MapEntry;

extern crate rand;
use rand::Rng;

use std::collections::HashMap;
use std::hash::{Hash,Hasher};
use std::rc::Rc;
use std::fmt::Debug;
use std::iter::FromIterator;
use std::fmt;

use std::ops::Deref;

// @TODO Change IFn's name -- IFn is a function, not an IFn.
//       The body it executes just happens to be an the IFn.  
/// Represents any Value known to ClojureRS, by wrapping any Value known to ClojureRS;
/// an int, a symbol, a fn, and so on.  Some Values here are more specific than others;
/// I32 wraps any I32, but QuoteMacro specifically wraps the value for the quote macro, which
/// is a special case macro that has hardcoded behavior. 
#[derive(Debug,Clone)]
pub enum Value {
    I32(i32),
    Symbol(Symbol),
    IFn(Rc<dyn IFn>),
    //
    // Special case functions
    //
    
    // I don't know if this exists in any particular Lisp,
    // but it allows me to reach into our local environment through an invoke
    LexicalEvalFn,
    
    PersistentList(PersistentList),
    PersistentVector(PersistentVector),
    PersistentListMap(PersistentListMap),
    
    Condition(std::string::String),
    // Macro body is still a function, that will be applied to our unevaled arguments 
    Macro(Rc<dyn IFn>),
    //
    // Special case macros 
    //
    QuoteMacro,
    DefmacroMacro,
    DefMacro,
    FnMacro,
    LetMacro,
   
    String(std::string::String),
    Nil
}
use crate::value::Value::*;

// @TODO since I have already manually defined hash,  surely this should just be defined
//       in terms of that?
impl PartialEq for Value {
    // @TODO derive from Hash in some way?  Note; can't derive with derive because of
    //       our trait objects in IFn and Macro
    // @TODO implement our generic IFns some other way? After all, again, this isn't Java 
    fn eq(&self, other: &Value) -> bool {
	// 
	if let I32(i) = self {
	    if let I32(i2) = other {
		return i == i2 
	    }    
	}

	if let Symbol(sym) = self {
	    if let Symbol(sym2) = other {
		return sym == sym2;
	    }
	}
	// Equality not defined on functions, similar to Clojure
	// Change this perhaps? Diverge?
	if let IFn(ifn) = self {
	    if let IFn(ifn2) = other {
		return false;
	    }
	}
	// Is it misleading for equality to sometimes work?
	if let LexicalEvalFn = self {
	    if let LexicalEvalFn = other {
		return true;
	    }
	}

	if let PersistentList(plist) = self {
	    if let PersistentList(plist2) = other {
		return plist == plist2;
	    }
	}

	if let PersistentVector(pvector) = self {
	    if let PersistentVector(pvector2) = other {
		return *pvector == *pvector2;
	    }
	}

	if let PersistentListMap(plistmap) = self {
	    if let PersistentListMap(plistmap2) = other {
		return *plistmap == *plistmap2;
	    }
	}

	if let Condition(msg) = self {
	    if let Condition(msg2) = other {
		return msg == msg2;
	    }
	}

	if let QuoteMacro = self {
	    if let QuoteMacro = other {
		return true;
	    }
	}

	if let DefmacroMacro = self {
	    if let DefmacroMacro = other {
		return true;
	    }
	}

	if let DefMacro = self {
	    if let DefMacro = other {
		return true;
	    }
	}

	if let LetMacro = self {
	    if let LetMacro = other {
		return true;
	    }
	}

	if let String(string) = self {
	    if let String(string2) = other {
		return string == string2;
	    }
	}

	if let Nil = self {
	    if let Nil = other {
		return true;
	    }
	}
	
	false
	
    }
}

// Again, this is certainly not the right away to do this
// @FIXME remove this entire monstrocity 
#[derive(Debug,Clone,Hash)]
enum ValueHash {
    LexicalEvalFn,
    QuoteMacro,
    DefmacroMacro,
    DefMacro,
    FnMacro,
    LetMacro,
    Nil
}
impl Eq for Value {}
impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
	match self {
	    I32(i) => i.hash(state),
	    Symbol(sym) => sym.hash(state),
	    IFn(_) => {
		let mut rng = rand::thread_rng();
		let n2: u16 = rng.gen();
		n2.hash(state)
	    },
	    LexicalEvalFn => (ValueHash::LexicalEvalFn).hash(state),
	    PersistentList(plist) => plist.hash(state),
	    PersistentVector(pvector) => pvector.hash(state),
	    PersistentListMap(plistmap) => plistmap.hash(state),
	    Condition(msg) => msg.hash(state),
	    // Random hash is temporary;
	    // @TODO implement hashing for functions / macros 
	    Macro(_) => {
		let mut rng = rand::thread_rng();
		let n2: u16 = rng.gen();
		n2.hash(state)
	    },
	    QuoteMacro => ValueHash::QuoteMacro.hash(state),
	    DefmacroMacro => ValueHash::DefmacroMacro.hash(state),
	    DefMacro => ValueHash::DefMacro.hash(state),
	    FnMacro => ValueHash::FnMacro.hash(state),
	    LetMacro => ValueHash::LetMacro.hash(state),

	    String(string) => string.hash(state),
	    Nil => ValueHash::Nil.hash(state),
	}
         // self.id.hash(state);
         // self.phone.hash(state);
     }
}
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	let str = match self {
	    I32(val) => val.to_string(),
	    Symbol(sym) => sym.to_string(),
	    IFn(_) => std::string::String::from("#function[]"),
	    LexicalEvalFn => std::string::String::from("#function[lexical-eval*]"),
	    PersistentList(plist) => plist.to_string(),
	    PersistentVector(pvector) => pvector.to_string(),
	    PersistentListMap(plistmap) => plistmap.to_string(),
	    Condition(msg) => format!("#Condition[\"{}\"]",msg),
	    Macro(_) => std::string::String::from("#macro[]"),
	    QuoteMacro => std::string::String::from("#macro[quote*]"),
	    DefMacro => std::string::String::from("#macro[def*]"),
	    DefmacroMacro => std::string::String::from("#macro[defmacro*]"),
	    FnMacro => std::string::String::from("#macro[fn*]"),
	    LetMacro => std::string::String::from("#macro[let*]"),
	    Value::String(string) => string.clone(),
	    Nil => std::string::String::from("nil"),
	};
	write!(f, "{}", str)
    }
}
impl Value {
    //
    // Likely temporary
    // I cannot remember for the life of me whether or not there's a function like this normally
    // and what its called
    // Regardless, when we have, say, a string inside a list, we want to print the string explicitly
    // with a \"\" and all.  
    // Everything else we print as is.
    //
    pub fn to_string_explicit(&self) -> std::string::String {
	match self {
	    Value::String(string) => format!("\"{}\"",string),
	    _ => self.to_string()
	}
    }
    pub fn type_tag(&self) -> TypeTag {
        match self {
            Value::I32(_) => TypeTag::I32,
            Value::Symbol(_) => TypeTag::Symbol,
            Value::IFn(_) => TypeTag::IFn,
	    Value::LexicalEvalFn => TypeTag::IFn,
            Value::PersistentList(_) => TypeTag::PersistentList,
	    Value::PersistentVector(_) => TypeTag::PersistentVector,
	    Value::PersistentListMap(_) => TypeTag::PersistentListMap,
            Value::Condition(_) => TypeTag::Condition,
            // Note; normal Clojure cannot take the value of a macro, so I don't imagine this
	    // having significance in the long run, but we will see 
	    Value::Macro(_) => TypeTag::Macro,
	    Value::QuoteMacro => TypeTag::Macro,
	    Value::DefMacro => TypeTag::Macro,
	    Value::DefmacroMacro => TypeTag::Macro,
	    Value::LetMacro => TypeTag::Macro,
	    Value::FnMacro => TypeTag::Macro,
	    Value::String(_) => TypeTag::String,
            Value::Nil => TypeTag::Nil 

        }
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////////////
    //
    // Eval Helper function
    //
    //////////////////////////////////////////////////////////////////////////////////////////////////////
    
    //
    // This function is inherently long, as it is dispatching on all valid function-like (IFn) Value types
    // We could further separate each dispatch case into individual functions, but I don't think that's necessary;
    // its not code that will be reused, and it doesn't make the code inherently shorter, it just moves it around
    // In this case, though, I don't find its movement to be one that would increase clarity;
    // this used to be a part of the overall eval function, and I think that DID
    // obscure a bit of clarity, because it added this extra level of nesting as a huge block in the middle
    // of the function,  and you could no longer just grok the functions shape at a glance,
    // nor could you know right away just by looking at the function what nested level of logic you were in.
    //
    // But now that this is separate, its only one level -- its only a list of implementations for each
    // IFn application, it might as well be a list of functions itself.  It in fact means you don't have to
    // hunt around for each individual implementation.  
    //
    /// Applies any valid function-like Value to a PersistentList, or returns None if our Value can't be applied
    fn apply_to_persistent_list(&self,environment: &Rc<Environment>,args: &Rc<PersistentList>) -> Option<Rc<Value>> {
	match self { 
	     Value::IFn(ifn) => {
		// Eval arguments 
		let evaled_arg_values = PersistentList::iter(args).map(|rc_arg| {
                    rc_arg.eval(Rc::clone(environment))
		}).collect::<Vec<Value>>();
		// Collect references for invoke 
		let evaled_args_refs = evaled_arg_values.iter().map(|arg| {
                    arg
		}).collect::<Vec<&Value>>();
		// Invoke fn on arguments 
		Some(Rc::new(ifn.invoke(evaled_args_refs)))
             },
	    LexicalEvalFn => {
		if args.len() != 1 {
		    return Some(Rc::new(Value::Condition(format!("Wrong number of arguments (Given: {}, Expected: 1)",args.len()))));
		}
		// This should only be one value
		let evaled_arg_values = PersistentList::iter(args).map(|rc_arg| {
                    rc_arg.eval(Rc::clone(environment))
		}).collect::<Vec<Value>>();

		let evaled_arg = evaled_arg_values.get(0).unwrap();
		
		Some(evaled_arg.eval_to_rc(Rc::clone(environment)))
	    },
	    //
	    // Unless I'm mistaken, this is incorrect; instead of having a phase where
	    // the macro expands, and then another phase where the whole expanded form
	    // is evaluated, it all happens at once.  I will have to look further into
	    // whether or not this will cause any problems; you'd think I'd know more
	    // about this particular step by now, but this is an implementation detail
	    // that's never interested me all that much
	    //
            Value::Macro(ifn) => {
		// Copy the args of the form into a new vector; these new values
		// will be used by the macro to create the new expanded form
		// (@TODO just reference the original args of the list if you can,
		//  since they're not mutated)
                let arg_values = PersistentList::iter(args).map(|rc_arg| {
                    (*rc_arg).clone()
                }).collect::<Vec<Value>>();

                
                let arg_refs =  arg_values.iter().map(|arg| {
                    arg
                }).collect::<Vec<&Value>>();

                let macroexpansion = Rc::new(ifn.invoke(arg_refs));

		Some(macroexpansion.eval_to_rc(Rc::clone(environment)))
		    
            },
	    //
	    // Special case macros 
	    //
	    // How these are implemented may change when we redesign macros
	    // That being said,  each of these macros introduce a various constraint
	    // that makes it easier to hardcode them into the evaluation step
	    // (or, for some, surely impossible not to do otherwise)
	    
	    //
	    // def is a primitive for modifying the environment itself,
	    // and it is easier to access the environment during this step,
	    // rather than owning some sort of reference to it in our def macro
	    // Edit:
	    //   The environment has now been modified to make it easy to close
	    //   around :D. Originally needed for our lambdas,  we can probably now,
	    //   should we choose,  define def without a 'special case macro', an extra
	    //   value type -- although we still need to hardcode its definition in Rust,
	    //   as an implementation of the generic Value::Macro(Rc<IFn>) 
	    //
	    DefMacro => {
		let arg_rc_values = PersistentList::iter(args).map(|rc_arg| {
		    rc_arg
                }).collect::<Vec<Rc<Value>>>();
		
		if arg_rc_values.len() > 2 || arg_rc_values.is_empty()  {
		    return Some(Rc::new(Value::Condition(format!("Wrong number of arguments (Given: {}, Expected: 1-2)",arg_rc_values.len()))));
		}
		let defname = arg_rc_values.get(0).unwrap();
		let defval = arg_rc_values.get(1).unwrap().eval_to_rc(Rc::clone(&environment));
		// Let's not do docstrings yet 
		// let docstring = ...
		match &**defname {
		    Value::Symbol(sym) => {
			environment.insert(sym.clone(),defval);
			// @TODO return var. For now, however, we only have symbols
			// @TODO intern from environment, don't make new sym ?
			Some(sym.to_rc_value())
		    },
		    _ => Some(Rc::new(Value::Condition(std::string::String::from("First argument to def must be a symbol"))))
		}
	    },
	    DefmacroMacro => {
		let arg_rc_values = PersistentList::iter(args).map(|rc_arg| {
		    rc_arg
                }).collect::<Vec<Rc<Value>>>();
		
		if arg_rc_values.len() < 2 || arg_rc_values.is_empty() {
		    return Some(Rc::new(Value::Condition(format!("Wrong number of arguments (Given: {}, Expected: >=2)",args.len()))))
		}
		let macro_name = arg_rc_values.get(0).unwrap();
		let macro_args = arg_rc_values.get(1).unwrap();

		let macro_body_exprs =
		    if arg_rc_values.len() <= 2 {
			&[]
		    } else { 
			arg_rc_values.get(2..).unwrap()
		    };
		let mut macro_invokable_body_vec = vec![
		    Symbol::intern("fn").to_rc_value(),
		    Rc::clone(macro_args)
		];
		// vec![do expr1 expr2 expr3]
		macro_invokable_body_vec.extend_from_slice(macro_body_exprs);
		let macro_invokable_body = macro_invokable_body_vec.into_list().eval(Rc::clone(&environment));
		let macro_value = match &macro_invokable_body {
		    Value::IFn(ifn) => Rc::new(Value::Macro(Rc::clone(&ifn))),
		    _ => Rc::new(Value::Condition(std::string::String::from("Compiler Error: your macro_value somehow compiled into something else entirely.  I don't even know how that happened,  this behavior is hardcoded, that's impressive")))
		};
		Some(vec![
		    Symbol::intern("def").to_rc_value(),
		    Rc::clone(macro_name),
		    macro_value
		].into_list().eval_to_rc(Rc::clone(&environment)))
	    },
	    //
	    // (fn [x y z] (+ x y z)) 
	    //
	    // @TODO Rename for* everywhere, define for in terms of for* in
	    //       ClojureRS
	    FnMacro => {
		let arg_rc_values = PersistentList::iter(args).map(|rc_arg| {
		    rc_arg
                }).collect::<Vec<Rc<Value>>>();
		
		if arg_rc_values.len() < 1 {
		    return Some(Rc::new(Value::Condition(format!("Wrong number of arguments (Given: {}, Expect: >=1",arg_rc_values.len()))));
		}
		// Let's not do fn names yet 
		// let fnname = arg_rc_values.get(0).unwrap();
		let fn_args = arg_rc_values.get(0).unwrap();
		// Let's not do docstrings yet 
		// let docstring = ...
		match &**fn_args {
		    Value::PersistentVector(PersistentVector{vals}) => {
			let mut arg_syms_vec = vec![];
			let enclosing_environment =
			    Rc::new(Environment::new_local_environment(Rc::clone(&environment)));
			for val in vals.iter() {
			    if let Value::Symbol(sym) = &**val {
				arg_syms_vec.push(sym.clone());
			    }
			}
			
			let fn_body =
			// (fn [x y] ) -> nil 
			    if arg_rc_values.len() <= 1 {
				Rc::new(Value::Nil)
				// (fn [x y] expr) -> expr 
			    } else if arg_rc_values.len() == 2 {
				Rc::clone(arg_rc_values.get(1).unwrap())
				// (fn [x y] expr1 expr2 expr3) -> (do expr1 expr2 expr3) 
			    } else {
				// (&[expr1 expr2 expr3] 
				let body_exprs = arg_rc_values.get(1..).unwrap();
				// vec![do]
				let mut do_body = vec![Symbol::intern("do").to_rc_value()];
				// vec![do expr1 expr2 expr3]
				do_body.extend_from_slice(body_exprs);
				// (do expr1 expr2 expr3) 
				do_body.into_list().to_rc_value()
			    };
			
			Some(Rc::new(lambda::Fn{
			    body: fn_body,
			    enclosing_environment,
			    arg_syms: arg_syms_vec
			}.to_value()))
		    },
		    _ => Some(Rc::new(Value::Condition(std::string::String::from("First argument to def must be a symbol"))))
		}
	    },
	    LetMacro => {
		let arg_rc_values = PersistentList::iter(args).map(|rc_arg| {
		    rc_arg
                }).collect::<Vec<Rc<Value>>>();
		if arg_rc_values.len() < 1 || arg_rc_values.len() > 2 {
		    return Some(Rc::new(Value::Condition(std::string::String::from("Wrong number of arguments given to let (Given: 0, Expecting: 1 or 2)"))));
		}
		// Already guaranteed to exist by earlier checks 
		let local_bindings = arg_rc_values.get(0).unwrap();
		match &**local_bindings {
		    Value::PersistentVector(vector) => {
			//let mut local_environment_map : HashMap<Symbol,Rc<Value>> = HashMap::new();
			let local_environment = Rc::new(Environment::new_local_environment(Rc::clone(environment)));
			// let chunk_test2 = 
			for pair in vector.vals.chunks(2) {
			    if let Some(rc_sym) = (&*pair).get(0) //(*pair[0]).clone()
			    {
				let val =
				    (&*pair).get(1).unwrap().eval_to_rc(Rc::clone(&local_environment));
				if let Value::Symbol(sym) = &(**rc_sym) {
				    local_environment.insert(sym.clone(),val);
				    //println!("Sym found: {:?}: {:?}",sym,val)
				}
			    }
			    else {
				//println!("Nope; pair: {:?}",pair)
			    }
			}
			let body = arg_rc_values.get(1);
			if let Some(body_) = body {
			    Some(body_.eval_to_rc(local_environment))
			}
			else {
			    Some(Rc::new(Value::Nil))
			}
		    },
		    _ => Some(Rc::new(Value::Condition(std::string::String::from("Bindings to let should be a vector"))))
		} 
	    },
	    // 
	    // Quote is simply a primitive, a macro base case; trying to define quote without
	    // quote just involves an infinite loop of macroexpansion. Or so it seems 
	    // 
	    QuoteMacro => {
		if args.len() > 1 {
		    Some(Rc::new(Value::Condition(format!("Wrong number of arguments (Given: {}, Expected: 1)",args.len()))))
		}
		// @TODO define is_empty()
		else if args.len() < 1 {
		    Some(Rc::new(Value::Condition(std::string::String::from("Wrong number of arguments (Given: 0, Expected: 1)"))))
		}
		else {
		    Some(args.nth(0))
		}
	    },
	    //
	    // If we're not a valid IFn 
	    //
	    _ => None 
	}
    }
    ////////////////////////////////////////////////////////////////////////////////////////////////////
    // Eval Helper
    ////////////////////////////////////////////////////////////////////////////////////////////////////
   
}
pub trait ToValue {
    fn to_value(&self) -> Value;
    fn to_rc_value(&self) -> Rc<Value>{
        Rc::new(self.to_value())
    }
}
impl ToValue for Value {
    fn to_value(&self) -> Value {
       self.clone()
    }
}
impl ToValue for Rc<Value> {
    fn to_value(&self) -> Value {
        (**self).clone()
    }
}
impl ToValue for i32 {
    fn to_value(&self) -> Value {
        Value::I32(*self) 
    }
}
impl ToValue for std::string::String {
    fn to_value(&self) -> Value {
        Value::String(self.clone()) 
    }
}
impl ToValue for str {
    fn to_value(&self) -> Value {
        Value::String(std::string::String::from(self)) 
    }
}
impl ToValue for Symbol {
    fn to_value(&self) -> Value {
        Value::Symbol(self.clone())
    }
}
impl ToValue for Rc<dyn IFn> {
    fn to_value(&self) -> Value {
        Value::IFn(Rc::clone(self))
    }
}
impl ToValue for PersistentList {
    fn to_value(&self) -> Value {
        Value::PersistentList(self.clone())
    }
}
impl ToValue for PersistentVector {
    fn to_value(&self) -> Value {
        Value::PersistentVector(self.clone())
    }
}
impl ToValue for PersistentListMap {
    fn to_value(&self) -> Value {
        Value::PersistentListMap(self.clone())
    }
}

/// Allows a type to be evaluated, abstracts evaluation
///
/// Our 'Value' type currently wraps and unites all types that exist within ClojureRS,
/// and therefore all values that are evaluated within ClojureRS,  so when not called on a Value,
/// this mostly acts as a shortcut for evaluating outside types not yet converted into a Value,
/// so you can write something like "1.eval(env)" instead of "1.to_value().eval(env)"
pub trait Evaluable {
    /// Evaluates a value and returns a Rc pointer to the evaluated ClojureRS Value 
    /// The program primarily
    fn eval_to_rc(&self,environment: Rc<Environment>) -> Rc<Value>;
    /// Evaluates a value and returns a new ClojureRS Value altogether, by cloning what
    /// eval_to_rc points to 
    fn eval(&self,environment: Rc<Environment> ) -> Value {
        self.eval_to_rc(environment).to_value() 
    }
}

impl Evaluable for Rc<Value> {

    fn eval_to_rc(&self, environment: Rc<Environment>) -> Rc<Value> {
        match &**self {
	    // Evaluating a symbol means grabbing the value its been bound to in our environment
            Value::Symbol(symbol) => environment.get(symbol),
	    // Evaluating a vector [a b c] just means [(eval a) (eval b) (eval c)]
	    Value::PersistentVector(pvector) => {
		// Evaluate each Rc<Value> our PersistentVector wraps
		// and return a new PersistentVector wrapping the new evaluated Values 
		let evaled_vals =  pvector.vals.iter().map(|rc_val| {
                    rc_val.eval_to_rc(Rc::clone(&environment))
                }).collect::<PersistentVector>();
		Rc::new(Value::PersistentVector(evaled_vals))
	    },
	    Value::PersistentListMap(plistmap) => {
		// Evaluate each Rc<Value> our PersistentVector wraps
		// and return a new PersistentVector wrapping the new evaluated Values 
		let evaled_vals =  plistmap.iter().map(|map_entry| {
		    MapEntry { key: map_entry.key.eval_to_rc(Rc::clone(&environment)),
			       val: map_entry.val.eval_to_rc(Rc::clone(&environment))}
                }).collect::<PersistentListMap>();
		Rc::new(Value::PersistentListMap(evaled_vals))
	    },
	    // Evaluating a list (a b c) means calling a as a function or macro on arguments b and c 
            Value::PersistentList(plist) => match plist {
                Cons(head,tail,__count) => {
		    // First we have to evaluate the head of our list and make sure it is function-like
		    // and can be invoked on our arguments
		    // (ie, a fn, a macro, a keyword ..)
		    // @TODO remove clone if possible 
                    let ifn = Rc::clone(head).eval_to_rc(Rc::clone(&environment));

		    let try_apply_ifn = ifn.apply_to_persistent_list(&Rc::clone(&environment),tail);

		    // Right now we're using the normal error message, however maybe later we will try
		    // 
		    // You tried to call value of type {} like a function, but only types of the
		    // interface clojure.lang.IFn can be called this way
		    //
		    // Sounds less correct but also seems clearer; the current error message relies on
		    // you pretty much already knowing when this error message is called
		    try_apply_ifn.unwrap_or(Rc::new(Value::Condition(format!("Execution Error: {} cannot be cast to clojure.lang.IFn",ifn.type_tag()))))
		},
		// () evals to () 
		PersistentList::Empty => Rc::new(Value::PersistentList(PersistentList::Empty))
            },
	    // Other types eval to self; (5 => 5,  "cat" => "cat",  #function[+] => #function[+]
            _ => Rc::clone(&self),
        }
    }
}
impl Evaluable for PersistentList {
    fn eval_to_rc(&self, environment: Rc<Environment>) -> Rc<Value> {
        self.to_rc_value().eval_to_rc(environment)
    }
}
impl Evaluable for Value {
    fn eval_to_rc(&self, environment: Rc<Environment>) -> Rc<Value> {
        self.to_rc_value().eval_to_rc(environment)
    }
}
