{
  zerofs,
}:

zerofs.overrideAttrs (oldAttrs: {
  patches = oldAttrs.patches ++ [
    ./hack.patch
  ];
})
