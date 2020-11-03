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
          {{name}}
        </template>
        <b-dropdown-item disabled>{{email}}</b-dropdown-item>
        <b-dropdown-divider></b-dropdown-divider>
        <b-dropdown-item href="#">My Profile</b-dropdown-item>
        <b-dropdown-item href="#">Help</b-dropdown-item>
        <b-dropdown-divider></b-dropdown-divider>
        <b-dropdown-item @click="logout">Logout</b-dropdown-item>
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
import axios from "axios"

export default {
  name: "Header",
  components: {
    IconLogo
  },
  data() {
    return {
      name: "",
      email: "",
      time: "November 3rd 2020, 3:54:22 am"
    }
  },
  methods: {
    logout: function () {
      $cookies.remove("token")
      this.$router.push('/')
    }
  },
  async mounted() {
    let user_id = $cookies.get("token");
    try {
      let user_data = await axios.get(
        `http://localhost:3001/api/users/${user_id}`
      );
      this.name = user_data.data.first_name + " " + user_data.data.last_name;
      this.email = user_data.data.email;
    } catch (err) {
      console.log(err);
    }
  },
  created() {
    setInterval(() => {
      this.time = "November 3rd 2020, 3:54:22 am";
    }, 1000)
  }
};
</script>
