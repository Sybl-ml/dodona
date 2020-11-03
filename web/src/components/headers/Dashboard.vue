<template>
  <b-navbar>
    <b-navbar-brand to="/">
      <icon-logo width="5em" height="3em" :show_text="true" />
    </b-navbar-brand>
    <b-navbar-nav>
      <b-nav-item disabled> {{time}} </b-nav-item>
    </b-navbar-nav>
    <b-navbar-nav class="ml-auto">
      <b-nav-item disabled>Credits: £20.20</b-nav-item>
      <b-nav-item-dropdown text="User 1" right>
        <template #button-content>
          <img src="https://www.w3schools.com/w3images/avatar6.png" class="img-circle">
          Joe Biden
        </template>
          <b-dropdown-item href="#">My Profile</b-dropdown-item>
          <b-dropdown-item href="#">Help</b-dropdown-item>
          <b-dropdown-divider></b-dropdown-divider>
          <b-dropdown-item href="#">Logout</b-dropdown-item>
        </b-nav-item-dropdown>
    </b-navbar-nav>
  </b-navbar>
</template>

<style>

.img-circle {
  height: 2rem;
  border-radius: 50%;
}

</style>

<script>
import IconBase from '../IconBase'
import IconLogo from '../icons/IconLogo'

export default {
  name: "Header",
  components: {
    IconLogo
  },
  data() {
    return {
      time: "November 3rd 2020, 3:54:22 am"
    }
  },
  created() {
    setInterval(() => {
      this.time = luxon.DateTime.local();
    }, 1000)
  }
};
</script>
