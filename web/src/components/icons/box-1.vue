<template>
  <svg
    width="100"
    height="100"
    viewBox="0 0 100 100"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    style="width: 251px; height: 251px;"
  >
    <path
      class="line-box1 stroke2"
      d="M50.2715 15.8421L50.2715 40.1579M50.2715 40.1579L60 30.4728M50.2715 40.1579L40.75 30.4728"
      stroke="#000"
      stroke-width="2px"
      stroke-linecap="round"
    ></path>
    <path
      class="stroke1 fill1"
      d="M76.1125 86.1005L50.5 89L25.3854 86.1022C24.8808 86.0439 24.5 85.6167 24.5 85.1087V62.6746C24.5 62.2669 24.7475 61.9 25.1255 61.7474L50.5 51.5L76.3684 61.7497C76.7496 61.9008 77 62.2693 77 62.6794V85.1068C77 85.6156 76.618 86.0432 76.1125 86.1005Z"
      fill="#FFF"
      stroke-width="2px"
    ></path>
    <path
      class="stroke1 fill1"
      d="M50.5 89L76.1125 86.1005C76.618 86.0432 77 85.6156 77 85.1068V62.6794C77 62.2693 76.7496 61.9008 76.3684 61.7497L50.5 51.5M50.5 89L25.3854 86.1022C24.8808 86.0439 24.5 85.6167 24.5 85.1087V62.6746C24.5 62.2669 24.7475 61.9 25.1255 61.7474L50.5 51.5M50.5 89V51.5"
      stroke="#000"
      stroke-width="2px"
    ></path>
    <path
      class="stroke1 fill2"
      d="M51.018 51.7306C50.5994 51.0191 51.1745 50.1355 51.9945 50.2301L75.4623 52.938C75.7978 52.9767 76.091 53.1821 76.2421 53.4841L82.2764 65.5528C82.6088 66.2177 82.1253 67 81.382 67H60.5719C60.2176 67 59.8897 66.8125 59.71 66.507L51.018 51.7306Z"
      fill="rgba(251,140,56,1)"
      stroke="#000"
      stroke-width="2px"
    ></path>
    <path
      class="stroke1 fill2"
      d="M49.982 51.7306C50.4006 51.0191 49.8255 50.1355 49.0055 50.2301L25.5377 52.938C25.2022 52.9767 24.909 53.1821 24.7579 53.4841L18.7236 65.5528C18.3912 66.2177 18.8747 67 19.618 67H40.4281C40.7824 67 41.1103 66.8125 41.29 66.507L49.982 51.7306Z"
      fill="rgba(251,140,56,1)"
      stroke="#000"
      stroke-width="2px"
    ></path>
  </svg>
</template>

<style>
.line-box1 {
  animation: line-box1-move 1.6s infinite;
}
@keyframes line-box1-move {
  0% {
    transform: translate3d(0, -2px, 0);
  }
  50% {
    transform: translate3d(0, 2px, 0);
  }
  100% {
    transform: translate3d(0, -2px, 0);
  }
}
@media (prefers-reduced-motion: reduce) {
  .line-box1 {
    animation: none;
  }
}
</style>
