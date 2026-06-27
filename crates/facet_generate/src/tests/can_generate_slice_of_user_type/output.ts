type Seq<T> = T[];

export class Tag {
    constructor () {
    }
}

export class Video {
    constructor (public tags: Seq<Tag>) {
    }
}
