type float32 = number;
type float64 = number;
type int32 = number;
type int8 = number;
type ListTuple<T extends any[]> = Tuple<T>[];
type Optional<T> = T | null;
type Seq<T> = T[];
type str = string;

export class CustomType {
    constructor () {
    }
}

export class Types {
    constructor (public s: str, public static_s: str, public int8: int8, public float: float32, public double: float64, public array: Seq<str>, public fixed_length_array: ListTuple<[str]>, public dictionary: Map<str,int32>, public optional_dictionary: Optional<Map<str,int32>>, public custom_type: CustomType) {
    }
}
