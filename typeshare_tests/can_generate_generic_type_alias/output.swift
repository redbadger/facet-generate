import Foundation

typealias GenericTypeAlias<T> = [T]

typealias NonGenericAlias = GenericTypeAlias<String?>
