import React from "react"
import { Navbar, Nav,} from 'react-bootstrap';
import styled from "styled-components";
import {PrimaryButton} from './Buttons';
import Toggle from "./Toggler"

const ClearHeaderBar = styled(Navbar)`
	min-height: 4rem;
    background: linear-gradient(${({ theme }) => theme.body}, ${({ theme }) => theme.body} 90%, transparent 100%);
    transition: all 0.25s linear;
`;


const ClearHeader = ({theme, toggleTheme}) => {
        return (
		<ClearHeaderBar sticky="top">
    		<ClearHeaderBar.Collapse className="justify-content-end">
				<Toggle theme={theme} toggleTheme={toggleTheme} />
				<Nav>
					<Nav.Link href="/login">Sign In</Nav.Link>

				</Nav>
				<PrimaryButton variant="primary" href="/register">SIGN UP NOW</PrimaryButton>
			</ClearHeaderBar.Collapse>
  		</ClearHeaderBar>
        );
    };
export default ClearHeader;