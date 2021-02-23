export default {
  tableClass: "table table-striped",
  loadingClass: "loading",
  ascendingIcon: "bi bi-caret-up-fill",
  descendingIcon: "bi bi-caret-down-fill",
  handleIcon: " ",
  renderIcon(classes, options) {
    return `<i class="${classes.join(" ")}"></i>`;
  },
  pagination: {
    infoClass: "pull-left",
    wrapperClass: "vuetable-pagination pull-right",
    activeClass: "btn-primary",
    disabledClass: "disabled",
    pageClass: "btn btn-border",
    linkClass: "btn btn-border",
    icons: {
      first: "",
      prev: "",
      next: "",
      last: "",
    },
  },
};