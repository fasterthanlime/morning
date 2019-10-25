

type Reader interface {
	Read(p []byte) (int, error)
}

type File struct {
	// ...
}

func (f *File) Read(p []byte) (int, error) {
	return 0, "derp"
}

//

impl Reader for *File {
	func Read(p []byte) (int, error) {
		// "self" is in scope
		return 0, "derp"
	}
}
