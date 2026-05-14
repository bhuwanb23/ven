// Fixture: mixes declared, stdlib, and ghost imports.
//   fmt, net/http                  -> stdlib (no dot in first segment)
//   github.com/gin-gonic/gin       -> declared in go.mod (not a ghost)
//   github.com/sirupsen/logrus     -> ghost
package main

import (
	"fmt"
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/sirupsen/logrus"
)

func main() {
	_ = fmt.Sprintf("ven fixture")
	_ = http.StatusOK
	_ = gin.New
	_ = logrus.New
}
