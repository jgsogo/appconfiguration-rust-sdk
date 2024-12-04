# Contributing

Review the following guidelines for submitting questions, issues, or changes to this repository.

## Questions

If you have questions about using the IBM Cloud App Configuration SDKs, you can ask questions in
[Stack Overflow](https://stackoverflow.com/questions/tagged/ibm-appconfiguration). Be sure to include
the `ibm-appconfiguration` tags.

## Coding Style

The SDK follows the Rust standards that are enforced via pre-commit hooks.


### Pre-commit hooks

Before starting to develop, please install [pre-commit hooks](https://pre-commit.com/)
by executing the following command inside the repository:

```
pre-commit install
```

## Issues

If you encounter an issue or want to suggest any improvement, you're welcome to submit an
[issue](https://github.com/IBM/appconfiguration-rust-sdk/issues).
Before that, please search for similar issues. It's possible somebody has encountered this issue already.

## Pull Requests

If you want to contribute to the repository, just open a pull-request in GitHub with your
changes to the `main` branch.

# Legal

Each source file must include a license header for the Apache
Software License 2.0:

```
Copyright [yyyy] [name of copyright owner]

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

We have tried to make it as easy as possible to make contributions. This
applies to how we handle the legal aspects of contribution. We use the
same approach - the [Developer's Certificate of Origin 1.1 (DCO)](DCO1.1.txt) - that the Linux® Kernel [community](https://elinux.org/Developer_Certificate_Of_Origin)
uses to manage code contributions.

We simply ask that when submitting a patch for review, the developer
must include a sign-off statement in the commit message.

Here is an example Signed-off-by line, which indicates that the
submitter accepts the DCO:

```
Signed-off-by: John Doe <john.doe@example.com>
```

You can include this automatically when you commit a change to your
local git repository using the following command:

```
git commit -s
```
