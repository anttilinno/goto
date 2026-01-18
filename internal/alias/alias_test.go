package alias

import "testing"

func TestValidate(t *testing.T) {
	tests := []struct {
		name    string
		alias   string
		wantErr bool
	}{
		{"simple", "dev", false},
		{"with-hyphen", "my-project", false},
		{"with-underscore", "my_project", false},
		{"with-numbers", "project123", false},
		{"starts-with-number", "123project", false},
		{"empty", "", true},
		{"starts-with-hyphen", "-invalid", true},
		{"starts-with-underscore", "_invalid", true},
		{"has-space", "my project", true},
		{"has-dot", "my.project", true},
		{"has-slash", "my/project", true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := Validate(tt.alias)
			if (err != nil) != tt.wantErr {
				t.Errorf("Validate(%q) error = %v, wantErr %v", tt.alias, err, tt.wantErr)
			}
		})
	}
}
