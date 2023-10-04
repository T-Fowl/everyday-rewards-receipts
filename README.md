# everyday-rewards-receipts

Simple tool to download all receipts from [everyday rewards](https://www.woolworthsrewards.com.au/) for later processing.

This tool will download the pdf receipt as well as store the api response of the transaction alongside as json.  
This sidecar file is useful as it contains the itemisation for the receipt - negating the need to strip text from the pdfs.

## Installation

If you are a Rust developer you can install from crates.io

`cargo install everyday-rewards-receipts`


## Usage

```shell
> everyday-rewards-receipts --help
Usage: everyday-rewards-receipts [OPTIONS] --token <TOKEN>

Options:
      --token <TOKEN>  
  -o, --output <PATH>  [default: ./receipts]
  -h, --help           Print help information
  -V, --version        Print version information
```

#### Retrieving  your authentication token:
1. Add the following bookmark  
    ```javascript
    javascript:(function(){alert("Access Token: " + JSON.parse(sessionStorage.authStatusData).access_token)})();
    ```
2. Go to [everyday rewards](https://www.woolworthsrewards.com.au/)
3. Login
4. Click on the bookmark and note down the presented value

#### Output structure:
```
output/
    # Derived from the Activity feed in everyday rewards
    This_Month/
        id0.pdf
        id0.json
        ...
    August_2021/
        id1.pdf
        id1.json
        ...
    August_2022/
        id2.pdf
        id2.json
        ...
    ...
```

### Licence

```markdown
MIT License

Copyright (c) [2022] [Thomas Fowler]

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
