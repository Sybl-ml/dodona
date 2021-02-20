export default {
    tableClass: 'table table-striped',
    loadingClass: 'loading',
    ascendingIcon: 'fa fa-chevron-up',
    descendingIcon: 'fa fa-chevron-down',
    handleIcon: 'bi bi-menu-hamburger',
    renderIcon(classes, options) {
        return `<i class="${classes.join(' ')}"></span>`
      },
    pagination: {
      infoClass: 'pull-left',
      wrapperClass: 'vuetable-pagination pull-right',
      activeClass: 'btn-primary',
      disabledClass: 'disabled',
      pageClass: 'btn btn-border',
      linkClass: 'btn btn-border',
      icons: {
        first: '',
        prev: '',
        next: '',
        last: '',
      },
    },
  }